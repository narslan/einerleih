use std::sync::Arc;

use async_trait::async_trait;
use deadpool_postgres::Pool;
use uuid::Uuid;

use crate::{
    common::error::AppError,
    domains::booking::{
        domain::{
            model::{Booking, BookingStatus},
            repository::BookingRepository,
            service::BookingServiceTrait,
        },
        dto::booking_dto::{BookingDto, BookingFilterDto, CreateBookingDto, UpdateBookingDto},
        infra::impl_repository::BookingRepo,
    },
    domains::mailbox::{
        MailboxRepo, MailboxRepository, dto::mailbox_dto::CreateBookingRequestMailboxEntriesDto,
    },
    domains::notification::{
        NotificationOutboxRepo, NotificationOutboxRepository, NotificationServiceTrait,
        dto::notification_dto::CreateBookingRequestNotificationDto,
    },
};

#[derive(Clone)]
pub struct BookingService {
    pub pool: Pool,
    pub repo: Arc<dyn BookingRepository + Send + Sync>,
    pub mailbox_repo: Arc<dyn MailboxRepository + Send + Sync>,
    pub notification_repo: Arc<dyn NotificationOutboxRepository + Send + Sync>,
    pub notification_service: Arc<dyn NotificationServiceTrait>,
}

fn validate_date_order(
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
) -> Result<(), AppError> {
    if end_date < start_date {
        return Err(AppError::ValidationError(
            "end_date must be on or after start_date".into(),
        ));
    }

    Ok(())
}

#[async_trait]
impl BookingServiceTrait for BookingService {
    fn create_service(
        pool: Pool,
        notification_service: Arc<dyn NotificationServiceTrait>,
    ) -> Arc<dyn BookingServiceTrait> {
        Arc::new(Self {
            pool,
            repo: Arc::new(BookingRepo {}),
            mailbox_repo: Arc::new(MailboxRepo {}),
            notification_repo: Arc::new(NotificationOutboxRepo {}),
            notification_service,
        })
    }

    async fn get_bookings_for_article(
        &self,
        article_id: Uuid,
        filter: BookingFilterDto,
    ) -> Result<Vec<BookingDto>, AppError> {
        if !self
            .repo
            .article_exists(self.pool.clone(), article_id)
            .await
            .map_err(AppError::DatabaseError)?
        {
            return Err(AppError::NotFound("Article not found".into()));
        }

        if let (Some(start_date), Some(end_date)) = (filter.start_date, filter.end_date) {
            validate_date_order(start_date, end_date)?;
        }

        self.repo
            .find_by_article_id(self.pool.clone(), article_id, filter)
            .await
            .map(|bookings| bookings.into_iter().map(BookingDto::from).collect())
            .map_err(AppError::DatabaseError)
    }

    async fn get_booking(
        &self,
        article_id: Uuid,
        booking_id: Uuid,
    ) -> Result<BookingDto, AppError> {
        self.repo
            .find_by_id(self.pool.clone(), article_id, booking_id)
            .await
            .map_err(AppError::DatabaseError)?
            .map(BookingDto::from)
            .ok_or_else(|| AppError::NotFound("Booking not found".into()))
    }

    async fn create_booking(
        &self,
        article_id: Uuid,
        payload: CreateBookingDto,
    ) -> Result<BookingDto, AppError> {
        validate_date_order(payload.start_date, payload.end_date)?;
        let requester_id = payload
            .requested_by
            .ok_or_else(|| AppError::ValidationError("requested_by is required".into()))?;

        if !self
            .repo
            .article_exists(self.pool.clone(), article_id)
            .await
            .map_err(AppError::DatabaseError)?
        {
            return Err(AppError::NotFound("Article not found".into()));
        }

        let mut client = self.pool.get().await.map_err(AppError::DatabaseError)?;
        let mut tx = client.transaction().await.map_err(|err| {
            tracing::error!("Error starting booking creation transaction: {err}");
            AppError::InternalError
        })?;
        let booking_id = Uuid::new_v4();
        let mailbox_payload = CreateBookingRequestMailboxEntriesDto {
            booking_id,
            article_id,
            requester_id,
            requester_name: payload.requester_name.clone(),
            note: payload.note.clone(),
            created_by: payload.created_by,
        };
        let notification_payload = CreateBookingRequestNotificationDto {
            booking_id,
            article_id,
            requester_name: payload.requester_name.clone(),
            note: payload.note.clone(),
            created_by: Some(payload.created_by),
        };

        if let Err(err) = self
            .repo
            .create(&mut tx, booking_id, article_id, payload)
            .await
        {
            tracing::error!("Error creating booking: {err}");
            let _ = tx.rollback().await;
            return Err(AppError::DatabaseError(err));
        }

        if let Err(err) = self
            .mailbox_repo
            .create_booking_request_entries(&mut tx, mailbox_payload)
            .await
        {
            tracing::error!("Error creating mailbox entries for booking request: {err}");
            let _ = tx.rollback().await;
            return Err(AppError::DatabaseError(err));
        }

        if let Err(err) = self
            .notification_repo
            .enqueue_booking_request(&mut tx, notification_payload)
            .await
        {
            tracing::error!("Error enqueuing booking request notification: {err}");
            let _ = tx.rollback().await;
            return Err(AppError::DatabaseError(err));
        }

        tx.commit().await.map_err(|err| {
            tracing::error!("Error committing booking creation: {err}");
            AppError::InternalError
        })?;

        if let Err(err) = self.notification_service.dispatch_pending(10).await {
            tracing::error!("Error dispatching queued notifications after booking creation: {err}");
        }

        self.get_booking(article_id, booking_id).await
    }

    async fn update_booking(
        &self,
        article_id: Uuid,
        booking_id: Uuid,
        payload: UpdateBookingDto,
    ) -> Result<BookingDto, AppError> {
        validate_date_order(payload.start_date, payload.end_date)?;
        let existing = find_existing_booking(self, article_id, booking_id).await?;

        if existing.status == BookingStatus::Confirmed {
            ensure_can_confirm_booking(
                self,
                article_id,
                Some(booking_id),
                payload.start_date,
                payload.end_date,
            )
            .await?;
        }

        let mut client = self.pool.get().await.map_err(AppError::DatabaseError)?;
        let mut tx = client.transaction().await.map_err(|err| {
            tracing::error!("Error starting booking update transaction: {err}");
            AppError::InternalError
        })?;

        match self
            .repo
            .update(&mut tx, article_id, booking_id, payload)
            .await
        {
            Ok(Some(booking)) => {
                tx.commit().await.map_err(|err| {
                    tracing::error!("Error committing booking update: {err}");
                    AppError::InternalError
                })?;
                Ok(BookingDto::from(booking))
            }
            Ok(None) => {
                let _ = tx.rollback().await;
                Err(AppError::NotFound("Booking not found".into()))
            }
            Err(err) => {
                tracing::error!("Error updating booking: {err}");
                let _ = tx.rollback().await;
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn confirm_booking(
        &self,
        article_id: Uuid,
        booking_id: Uuid,
        actor_id: Uuid,
    ) -> Result<BookingDto, AppError> {
        let booking = find_existing_booking(self, article_id, booking_id).await?;
        ensure_can_confirm_booking(
            self,
            article_id,
            Some(booking_id),
            booking.start_date,
            booking.end_date,
        )
        .await?;

        update_booking_status(
            self,
            article_id,
            booking_id,
            BookingStatus::Confirmed,
            actor_id,
        )
        .await
    }

    async fn reject_booking(
        &self,
        article_id: Uuid,
        booking_id: Uuid,
        actor_id: Uuid,
    ) -> Result<BookingDto, AppError> {
        update_booking_status(
            self,
            article_id,
            booking_id,
            BookingStatus::Rejected,
            actor_id,
        )
        .await
    }

    async fn cancel_booking(
        &self,
        article_id: Uuid,
        booking_id: Uuid,
        actor_id: Uuid,
    ) -> Result<BookingDto, AppError> {
        update_booking_status(
            self,
            article_id,
            booking_id,
            BookingStatus::Cancelled,
            actor_id,
        )
        .await
    }

    async fn complete_booking(
        &self,
        article_id: Uuid,
        booking_id: Uuid,
        actor_id: Uuid,
    ) -> Result<BookingDto, AppError> {
        update_booking_status(
            self,
            article_id,
            booking_id,
            BookingStatus::Completed,
            actor_id,
        )
        .await
    }
}

async fn find_existing_booking(
    service: &BookingService,
    article_id: Uuid,
    booking_id: Uuid,
) -> Result<Booking, AppError> {
    service
        .repo
        .find_by_id(service.pool.clone(), article_id, booking_id)
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Booking not found".into()))
}

async fn ensure_can_confirm_booking(
    service: &BookingService,
    article_id: Uuid,
    booking_id: Option<Uuid>,
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
) -> Result<(), AppError> {
    if service
        .repo
        .has_confirmed_booking_conflict(
            service.pool.clone(),
            article_id,
            booking_id,
            start_date,
            end_date,
        )
        .await
        .map_err(AppError::DatabaseError)?
    {
        return Err(AppError::ValidationError(
            "Booking overlaps an already confirmed booking".into(),
        ));
    }

    Ok(())
}

async fn update_booking_status(
    service: &BookingService,
    article_id: Uuid,
    booking_id: Uuid,
    status: BookingStatus,
    actor_id: Uuid,
) -> Result<BookingDto, AppError> {
    let mut client = service.pool.get().await.map_err(AppError::DatabaseError)?;
    let mut tx = client.transaction().await.map_err(|err| {
        tracing::error!("Error starting booking status transaction: {err}");
        AppError::InternalError
    })?;

    match service
        .repo
        .update_status(&mut tx, article_id, booking_id, status, actor_id)
        .await
    {
        Ok(Some(booking)) => {
            tx.commit().await.map_err(|err| {
                tracing::error!("Error committing booking status update: {err}");
                AppError::InternalError
            })?;
            Ok(BookingDto::from(booking))
        }
        Ok(None) => {
            let _ = tx.rollback().await;
            Err(AppError::NotFound("Booking not found".into()))
        }
        Err(err) => {
            tracing::error!("Error updating booking status: {err}");
            let _ = tx.rollback().await;
            Err(AppError::DatabaseError(err))
        }
    }
}
