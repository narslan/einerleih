use std::sync::Arc;

use async_trait::async_trait;
use deadpool_postgres::Pool;

use crate::{
    common::{config::Config, error::AppError},
    domains::notification::{
        EmailSender, FileEmailSender, NotificationOutboxRepo, NotificationOutboxRepository,
        NotificationServiceTrait,
        dto::notification_dto::{EnqueueEmailNotificationDto, NotificationDispatchResultDto},
    },
};

#[derive(Clone)]
pub struct NotificationService {
    pub pool: Pool,
    pub repo: Arc<dyn NotificationOutboxRepository + Send + Sync>,
    pub sender: Arc<dyn EmailSender + Send + Sync>,
}

#[async_trait]
impl NotificationServiceTrait for NotificationService {
    fn create_service(config: Config, pool: Pool) -> Arc<dyn NotificationServiceTrait> {
        Arc::new(Self {
            pool,
            repo: Arc::new(NotificationOutboxRepo {}),
            sender: Arc::new(FileEmailSender::new(config.notification_file_output_dir)),
        })
    }

    async fn enqueue_email(&self, payload: EnqueueEmailNotificationDto) -> Result<(), AppError> {
        if payload.recipient_email.trim().is_empty() {
            return Err(AppError::ValidationError(
                "recipient_email is required".into(),
            ));
        }

        if payload.subject.trim().is_empty() {
            return Err(AppError::ValidationError("subject is required".into()));
        }

        if payload.body_text.trim().is_empty() {
            return Err(AppError::ValidationError("body_text is required".into()));
        }

        let mut client = self.pool.get().await.map_err(AppError::DatabaseError)?;
        let mut tx = client.transaction().await.map_err(|err| {
            tracing::error!("Error starting notification enqueue transaction: {err}");
            AppError::InternalError
        })?;

        self.repo
            .enqueue(&mut tx, payload)
            .await
            .map_err(AppError::DatabaseError)?;

        tx.commit().await.map_err(|err| {
            tracing::error!("Error committing notification enqueue transaction: {err}");
            AppError::InternalError
        })?;

        Ok(())
    }

    async fn dispatch_pending(
        &self,
        limit: i64,
    ) -> Result<Vec<NotificationDispatchResultDto>, AppError> {
        let notifications = self
            .repo
            .find_dispatchable(self.pool.clone(), limit)
            .await
            .map_err(AppError::DatabaseError)?;

        let mut results = Vec::with_capacity(notifications.len());

        for notification in notifications {
            let send_result = self.sender.send(&notification).await;
            let mut client = self.pool.get().await.map_err(AppError::DatabaseError)?;
            let mut tx = client.transaction().await.map_err(|err| {
                tracing::error!("Error starting notification dispatch transaction: {err}");
                AppError::InternalError
            })?;

            match send_result {
                Ok(()) => {
                    self.repo
                        .mark_sent(&mut tx, notification.notification_id)
                        .await
                        .map_err(AppError::DatabaseError)?;
                    tx.commit().await.map_err(|err| {
                        tracing::error!("Error committing notification sent state: {err}");
                        AppError::InternalError
                    })?;
                    results.push(NotificationDispatchResultDto {
                        notification_id: notification.notification_id,
                        sent: true,
                        error: None,
                    });
                }
                Err(error_message) => {
                    self.repo
                        .mark_failed(&mut tx, notification.notification_id, &error_message)
                        .await
                        .map_err(AppError::DatabaseError)?;
                    tx.commit().await.map_err(|err| {
                        tracing::error!("Error committing notification failed state: {err}");
                        AppError::InternalError
                    })?;
                    results.push(NotificationDispatchResultDto {
                        notification_id: notification.notification_id,
                        sent: false,
                        error: Some(error_message),
                    });
                }
            }
        }

        Ok(results)
    }
}
