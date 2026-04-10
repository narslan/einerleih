use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::domains::calendar::domain::model::{
    CalendarBlockReason, CalendarEntry, CalendarEntrySource, CalendarEntryType,
};

#[derive(PartialEq, Debug, Deserialize, Serialize, ToSchema)]
pub struct CalendarEntryDto {
    pub event_id: uuid::Uuid,
    pub article_id: uuid::Uuid,
    pub entry_type: CalendarEntryType,
    pub block_reason: Option<CalendarBlockReason>,
    pub summary: String,
    pub location: Option<String>,
    pub description: Option<String>,
    #[serde(with = "crate::common::ts_format")]
    pub start_time: DateTime<Utc>,
    #[serde(with = "crate::common::ts_format")]
    pub end_time: DateTime<Utc>,
    pub rrule: Option<String>,
    #[serde(with = "crate::common::ts_format::option")]
    pub dtstamp: Option<DateTime<Utc>>,
    pub source: CalendarEntrySource,
    pub created_by: Option<uuid::Uuid>,
    #[serde(with = "crate::common::ts_format::option")]
    pub created_at: Option<DateTime<Utc>>,
    pub modified_by: Option<uuid::Uuid>,
    #[serde(with = "crate::common::ts_format::option")]
    pub modified_at: Option<DateTime<Utc>>,
}

#[derive(PartialEq, Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct CreateCalendarEntryDto {
    pub entry_type: CalendarEntryType,
    pub block_reason: Option<CalendarBlockReason>,
    #[validate(length(max = 255, message = "Summary cannot exceed 255 characters"))]
    pub summary: String,
    #[validate(length(max = 255, message = "Location cannot exceed 255 characters"))]
    pub location: Option<String>,
    pub description: Option<String>,
    #[serde(with = "crate::common::ts_format")]
    pub start_time: DateTime<Utc>,
    #[serde(with = "crate::common::ts_format")]
    pub end_time: DateTime<Utc>,
    pub rrule: Option<String>,
    #[serde(default)]
    pub source: CalendarEntrySource,
    #[serde(default)]
    pub created_by: uuid::Uuid,
    #[serde(default)]
    pub modified_by: uuid::Uuid,
}

#[derive(PartialEq, Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct UpdateCalendarEntryDto {
    pub entry_type: CalendarEntryType,
    pub block_reason: Option<CalendarBlockReason>,
    #[validate(length(max = 255, message = "Summary cannot exceed 255 characters"))]
    pub summary: String,
    #[validate(length(max = 255, message = "Location cannot exceed 255 characters"))]
    pub location: Option<String>,
    pub description: Option<String>,
    #[serde(with = "crate::common::ts_format")]
    pub start_time: DateTime<Utc>,
    #[serde(with = "crate::common::ts_format")]
    pub end_time: DateTime<Utc>,
    pub rrule: Option<String>,
    #[serde(default)]
    pub source: CalendarEntrySource,
    #[serde(default)]
    pub modified_by: uuid::Uuid,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct CalendarEntryFilterDto {
    #[serde(default, with = "crate::common::ts_format::option")]
    pub start_time: Option<DateTime<Utc>>,
    #[serde(default, with = "crate::common::ts_format::option")]
    pub end_time: Option<DateTime<Utc>>,
}

impl From<CalendarEntry> for CalendarEntryDto {
    fn from(value: CalendarEntry) -> Self {
        Self {
            event_id: value.event_id,
            article_id: value.article_id,
            entry_type: value.entry_type,
            block_reason: value.block_reason,
            summary: value.summary,
            location: value.location,
            description: value.description,
            start_time: value.start_time,
            end_time: value.end_time,
            rrule: value.rrule,
            dtstamp: value.dtstamp,
            source: value.source,
            created_by: value.created_by,
            created_at: value.created_at,
            modified_by: value.modified_by,
            modified_at: value.modified_at,
        }
    }
}
