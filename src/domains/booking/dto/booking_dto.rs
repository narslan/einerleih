use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::domains::booking::domain::model::{Booking, BookingStatus};

#[derive(PartialEq, Debug, Deserialize, Serialize, ToSchema)]
pub struct BookingDto {
    pub booking_id: uuid::Uuid,
    pub article_id: uuid::Uuid,
    pub requested_by: Option<uuid::Uuid>,
    pub requester_name: Option<String>,
    pub requester_email: Option<String>,
    pub note: Option<String>,
    #[serde(with = "crate::common::ts_format")]
    pub start_time: DateTime<Utc>,
    #[serde(with = "crate::common::ts_format")]
    pub end_time: DateTime<Utc>,
    pub status: BookingStatus,
    pub approved_by: Option<uuid::Uuid>,
    #[serde(with = "crate::common::ts_format::option")]
    pub approved_at: Option<DateTime<Utc>>,
    pub created_by: Option<uuid::Uuid>,
    #[serde(with = "crate::common::ts_format::option")]
    pub created_at: Option<DateTime<Utc>>,
    pub modified_by: Option<uuid::Uuid>,
    #[serde(with = "crate::common::ts_format::option")]
    pub modified_at: Option<DateTime<Utc>>,
}

#[derive(PartialEq, Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct CreateBookingDto {
    pub requested_by: Option<uuid::Uuid>,
    #[validate(length(max = 128, message = "Requester name cannot exceed 128 characters"))]
    pub requester_name: Option<String>,
    #[validate(email(message = "Requester email must be valid"))]
    #[validate(length(max = 128, message = "Requester email cannot exceed 128 characters"))]
    pub requester_email: Option<String>,
    pub note: Option<String>,
    #[serde(with = "crate::common::ts_format")]
    pub start_time: DateTime<Utc>,
    #[serde(with = "crate::common::ts_format")]
    pub end_time: DateTime<Utc>,
    #[serde(default)]
    pub created_by: uuid::Uuid,
    #[serde(default)]
    pub modified_by: uuid::Uuid,
}

#[derive(PartialEq, Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct UpdateBookingDto {
    pub requested_by: Option<uuid::Uuid>,
    #[validate(length(max = 128, message = "Requester name cannot exceed 128 characters"))]
    pub requester_name: Option<String>,
    #[validate(email(message = "Requester email must be valid"))]
    #[validate(length(max = 128, message = "Requester email cannot exceed 128 characters"))]
    pub requester_email: Option<String>,
    pub note: Option<String>,
    #[serde(with = "crate::common::ts_format")]
    pub start_time: DateTime<Utc>,
    #[serde(with = "crate::common::ts_format")]
    pub end_time: DateTime<Utc>,
    #[serde(default)]
    pub modified_by: uuid::Uuid,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct BookingFilterDto {
    #[serde(default, with = "crate::common::ts_format::option")]
    pub start_time: Option<DateTime<Utc>>,
    #[serde(default, with = "crate::common::ts_format::option")]
    pub end_time: Option<DateTime<Utc>>,
    pub status: Option<BookingStatus>,
}

impl From<Booking> for BookingDto {
    fn from(value: Booking) -> Self {
        Self {
            booking_id: value.booking_id,
            article_id: value.article_id,
            requested_by: value.requested_by,
            requester_name: value.requester_name,
            requester_email: value.requester_email,
            note: value.note,
            start_time: value.start_time,
            end_time: value.end_time,
            status: value.status,
            approved_by: value.approved_by,
            approved_at: value.approved_at,
            created_by: value.created_by,
            created_at: value.created_at,
            modified_by: value.modified_by,
            modified_at: value.modified_at,
        }
    }
}
