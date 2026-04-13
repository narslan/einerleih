use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::domains::calendar::domain::model::{CalendarEntry, CalendarEntrySource};

#[derive(PartialEq, Debug, Deserialize, Serialize, ToSchema)]
pub struct CalendarEntryDto {
    pub event_id: uuid::Uuid,
    pub article_id: uuid::Uuid,
    #[serde(with = "crate::common::ts_format::date")]
    pub start_date: chrono::NaiveDate,
    #[serde(with = "crate::common::ts_format::date")]
    pub end_date: chrono::NaiveDate,
    pub location: Option<String>,
    pub description: Option<String>,
    #[serde(with = "crate::common::ts_format::option")]
    pub dtstamp: Option<chrono::DateTime<chrono::Utc>>,
    pub source: CalendarEntrySource,
    pub created_by: Option<uuid::Uuid>,
    #[serde(with = "crate::common::ts_format::option")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub modified_by: Option<uuid::Uuid>,
    #[serde(with = "crate::common::ts_format::option")]
    pub modified_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(PartialEq, Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct CreateCalendarEntryDto {
    #[serde(with = "crate::common::ts_format::date")]
    pub start_date: chrono::NaiveDate,
    #[serde(with = "crate::common::ts_format::date")]
    pub end_date: chrono::NaiveDate,
    #[validate(length(max = 255, message = "Location cannot exceed 255 characters"))]
    pub location: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub source: CalendarEntrySource,
    #[serde(default)]
    pub created_by: uuid::Uuid,
    #[serde(default)]
    pub modified_by: uuid::Uuid,
}

#[derive(PartialEq, Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct UpdateCalendarEntryDto {
    #[serde(with = "crate::common::ts_format::date")]
    pub start_date: chrono::NaiveDate,
    #[serde(with = "crate::common::ts_format::date")]
    pub end_date: chrono::NaiveDate,
    #[validate(length(max = 255, message = "Location cannot exceed 255 characters"))]
    pub location: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub source: CalendarEntrySource,
    #[serde(default)]
    pub modified_by: uuid::Uuid,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct CalendarEntryFilterDto {
    #[serde(default, with = "crate::common::ts_format::date::option")]
    pub start_date: Option<chrono::NaiveDate>,
    #[serde(default, with = "crate::common::ts_format::date::option")]
    pub end_date: Option<chrono::NaiveDate>,
}

impl From<CalendarEntry> for CalendarEntryDto {
    fn from(value: CalendarEntry) -> Self {
        Self {
            event_id: value.event_id,
            article_id: value.article_id,
            start_date: value.start_date,
            end_date: value.end_date,
            location: value.location,
            description: value.description,
            dtstamp: value.dtstamp,
            source: value.source,
            created_by: value.created_by,
            created_at: value.created_at,
            modified_by: value.modified_by,
            modified_at: value.modified_at,
        }
    }
}
