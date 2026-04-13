use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::domains::mailbox::domain::model::{MailboxDirection, MailboxEntry};

#[derive(PartialEq, Debug, Deserialize, Serialize, ToSchema)]
pub struct MailboxEntryDto {
    pub mailbox_entry_id: uuid::Uuid,
    pub booking_id: Option<uuid::Uuid>,
    pub article_id: uuid::Uuid,
    pub owner_id: uuid::Uuid,
    pub sender_id: uuid::Uuid,
    pub recipient_id: uuid::Uuid,
    pub direction: MailboxDirection,
    pub subject: String,
    pub body: String,
    #[serde(with = "crate::common::ts_format::option")]
    pub read_at: Option<DateTime<Utc>>,
    pub created_by: Option<uuid::Uuid>,
    #[serde(with = "crate::common::ts_format::option")]
    pub created_at: Option<DateTime<Utc>>,
    pub modified_by: Option<uuid::Uuid>,
    #[serde(with = "crate::common::ts_format::option")]
    pub modified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct MailboxEntryFilterDto {
    pub direction: Option<MailboxDirection>,
}

#[derive(Debug, Clone)]
pub struct CreateBookingRequestMailboxEntriesDto {
    pub booking_id: uuid::Uuid,
    pub article_id: uuid::Uuid,
    pub requester_id: uuid::Uuid,
    pub requester_name: Option<String>,
    pub note: Option<String>,
    pub created_by: uuid::Uuid,
}

impl From<MailboxEntry> for MailboxEntryDto {
    fn from(value: MailboxEntry) -> Self {
        Self {
            mailbox_entry_id: value.mailbox_entry_id,
            booking_id: value.booking_id,
            article_id: value.article_id,
            owner_id: value.owner_id,
            sender_id: value.sender_id,
            recipient_id: value.recipient_id,
            direction: value.direction,
            subject: value.subject,
            body: value.body,
            read_at: value.read_at,
            created_by: value.created_by,
            created_at: value.created_at,
            modified_by: value.modified_by,
            modified_at: value.modified_at,
        }
    }
}
