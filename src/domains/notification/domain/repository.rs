use async_trait::async_trait;
use deadpool_postgres::{Pool, PoolError, Transaction};
use uuid::Uuid;

use crate::domains::notification::{
    domain::model::NotificationOutboxEntry,
    dto::notification_dto::{CreateBookingRequestNotificationDto, EnqueueEmailNotificationDto},
};

#[async_trait]
pub trait NotificationOutboxRepository: Send + Sync {
    async fn enqueue(
        &self,
        tx: &mut Transaction<'_>,
        payload: EnqueueEmailNotificationDto,
    ) -> Result<Uuid, PoolError>;

    async fn enqueue_booking_request(
        &self,
        tx: &mut Transaction<'_>,
        payload: CreateBookingRequestNotificationDto,
    ) -> Result<Option<Uuid>, PoolError>;

    async fn find_dispatchable(
        &self,
        pool: Pool,
        limit: i64,
    ) -> Result<Vec<NotificationOutboxEntry>, PoolError>;

    async fn mark_sent(
        &self,
        tx: &mut Transaction<'_>,
        notification_id: Uuid,
    ) -> Result<(), PoolError>;

    async fn mark_failed(
        &self,
        tx: &mut Transaction<'_>,
        notification_id: Uuid,
        error_message: &str,
    ) -> Result<(), PoolError>;
}
