use std::sync::Arc;

use async_trait::async_trait;
use deadpool_postgres::Pool;

use crate::{
    common::{config::Config, error::AppError},
    domains::notification::dto::notification_dto::{
        EnqueueEmailNotificationDto, NotificationDispatchResultDto,
    },
};

#[async_trait]
pub trait NotificationServiceTrait: Send + Sync {
    fn create_service(config: Config, pool: Pool) -> Arc<dyn NotificationServiceTrait>
    where
        Self: Sized;

    async fn enqueue_email(&self, payload: EnqueueEmailNotificationDto) -> Result<(), AppError>;

    async fn dispatch_pending(
        &self,
        limit: i64,
    ) -> Result<Vec<NotificationDispatchResultDto>, AppError>;
}
