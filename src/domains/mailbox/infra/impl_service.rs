use std::sync::Arc;

use async_trait::async_trait;
use deadpool_postgres::Pool;
use uuid::Uuid;

use crate::{
    common::error::AppError,
    domains::mailbox::{
        domain::{repository::MailboxRepository, service::MailboxServiceTrait},
        dto::mailbox_dto::{
            CreateBookingRequestMailboxEntriesDto, MailboxEntryDto, MailboxEntryFilterDto,
        },
        infra::impl_repository::MailboxRepo,
    },
};

#[derive(Clone)]
pub struct MailboxService {
    pub pool: Pool,
    pub repo: Arc<dyn MailboxRepository + Send + Sync>,
}

#[async_trait]
impl MailboxServiceTrait for MailboxService {
    fn create_service(pool: Pool) -> Arc<dyn MailboxServiceTrait> {
        Arc::new(Self {
            pool,
            repo: Arc::new(MailboxRepo {}),
        })
    }

    async fn get_entries(
        &self,
        owner_id: Uuid,
        filter: MailboxEntryFilterDto,
    ) -> Result<Vec<MailboxEntryDto>, AppError> {
        self.repo
            .find_by_owner_id(self.pool.clone(), owner_id, filter)
            .await
            .map(|entries| entries.into_iter().map(MailboxEntryDto::from).collect())
            .map_err(AppError::DatabaseError)
    }

    async fn get_entry(
        &self,
        owner_id: Uuid,
        mailbox_entry_id: Uuid,
    ) -> Result<MailboxEntryDto, AppError> {
        self.repo
            .find_by_owner_and_id(self.pool.clone(), owner_id, mailbox_entry_id)
            .await
            .map_err(AppError::DatabaseError)?
            .map(MailboxEntryDto::from)
            .ok_or_else(|| AppError::NotFound("Mailbox entry not found".into()))
    }

    async fn mark_as_read(
        &self,
        owner_id: Uuid,
        mailbox_entry_id: Uuid,
    ) -> Result<MailboxEntryDto, AppError> {
        let mut client = self.pool.get().await.map_err(AppError::DatabaseError)?;
        let mut tx = client.transaction().await.map_err(|err| {
            tracing::error!("Error starting mailbox read transaction: {err}");
            AppError::InternalError
        })?;

        match self
            .repo
            .mark_as_read(&mut tx, owner_id, mailbox_entry_id)
            .await
        {
            Ok(Some(entry)) => {
                tx.commit().await.map_err(|err| {
                    tracing::error!("Error committing mailbox read transaction: {err}");
                    AppError::InternalError
                })?;
                Ok(MailboxEntryDto::from(entry))
            }
            Ok(None) => {
                let _ = tx.rollback().await;
                Err(AppError::NotFound("Mailbox entry not found".into()))
            }
            Err(err) => {
                tracing::error!("Error marking mailbox entry as read: {err}");
                let _ = tx.rollback().await;
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn create_booking_request_entries(
        &self,
        payload: CreateBookingRequestMailboxEntriesDto,
    ) -> Result<(), AppError> {
        let mut client = self.pool.get().await.map_err(AppError::DatabaseError)?;
        let mut tx = client.transaction().await.map_err(|err| {
            tracing::error!("Error starting mailbox creation transaction: {err}");
            AppError::InternalError
        })?;

        if let Err(err) = self
            .repo
            .create_booking_request_entries(&mut tx, payload)
            .await
        {
            tracing::error!("Error creating booking mailbox entries: {err}");
            let _ = tx.rollback().await;
            return Err(AppError::DatabaseError(err));
        }

        tx.commit().await.map_err(|err| {
            tracing::error!("Error committing mailbox creation transaction: {err}");
            AppError::InternalError
        })
    }
}
