use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BookingStatus {
    Requested,
    Confirmed,
    Rejected,
    Cancelled,
    Completed,
}

impl BookingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Confirmed => "confirmed",
            Self::Rejected => "rejected",
            Self::Cancelled => "cancelled",
            Self::Completed => "completed",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "requested" => Some(Self::Requested),
            "confirmed" => Some(Self::Confirmed),
            "rejected" => Some(Self::Rejected),
            "cancelled" => Some(Self::Cancelled),
            "completed" => Some(Self::Completed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Booking {
    pub booking_id: uuid::Uuid,
    pub article_id: uuid::Uuid,
    pub requested_by: Option<uuid::Uuid>,
    pub requester_name: Option<String>,
    pub requester_email: Option<String>,
    pub note: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub status: BookingStatus,
    pub approved_by: Option<uuid::Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_by: Option<uuid::Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_by: Option<uuid::Uuid>,
    pub modified_at: Option<DateTime<Utc>>,
}
