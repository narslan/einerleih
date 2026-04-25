use uuid::Uuid;

use crate::domains::notification::domain::model::NotificationKind;

#[derive(Debug, Clone)]
pub struct EnqueueEmailNotificationDto {
    pub kind: NotificationKind,
    pub recipient_email: String,
    pub subject: String,
    pub body_text: String,
    pub booking_id: Option<Uuid>,
    pub article_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct CreateBookingRequestNotificationDto {
    pub booking_id: Uuid,
    pub article_id: Uuid,
    pub requester_name: Option<String>,
    pub note: Option<String>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct NotificationDispatchResultDto {
    pub notification_id: Uuid,
    pub sent: bool,
    pub error: Option<String>,
}
