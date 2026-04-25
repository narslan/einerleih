use async_trait::async_trait;

use crate::domains::notification::domain::model::NotificationOutboxEntry;

#[async_trait]
pub trait EmailSender: Send + Sync {
    async fn send(&self, notification: &NotificationOutboxEntry) -> Result<(), String>;
}
