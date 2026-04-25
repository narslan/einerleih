use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationKind {
    SignupConfirmation,
    BookingRequest,
}

impl NotificationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SignupConfirmation => "signup_confirmation",
            Self::BookingRequest => "booking_request",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "signup_confirmation" => Some(Self::SignupConfirmation),
            "booking_request" => Some(Self::BookingRequest),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationStatus {
    Pending,
    Sent,
    Failed,
}

impl NotificationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Sent => "sent",
            Self::Failed => "failed",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "sent" => Some(Self::Sent),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NotificationOutboxEntry {
    pub notification_id: Uuid,
    pub kind: NotificationKind,
    pub status: NotificationStatus,
    pub recipient_email: String,
    pub subject: String,
    pub body_text: String,
    pub booking_id: Option<Uuid>,
    pub article_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub attempt_count: i32,
    pub last_error: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
}
