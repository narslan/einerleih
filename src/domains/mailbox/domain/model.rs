use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum MailboxDirection {
    Inbox,
    Sent,
}

impl MailboxDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Inbox => "inbox",
            Self::Sent => "sent",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "inbox" => Some(Self::Inbox),
            "sent" => Some(Self::Sent),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MailboxEntry {
    pub mailbox_entry_id: uuid::Uuid,
    pub booking_id: Option<uuid::Uuid>,
    pub article_id: uuid::Uuid,
    pub owner_id: uuid::Uuid,
    pub sender_id: uuid::Uuid,
    pub recipient_id: uuid::Uuid,
    pub direction: MailboxDirection,
    pub subject: String,
    pub body: String,
    pub read_at: Option<DateTime<Utc>>,
    pub created_by: Option<uuid::Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_by: Option<uuid::Uuid>,
    pub modified_at: Option<DateTime<Utc>>,
}
