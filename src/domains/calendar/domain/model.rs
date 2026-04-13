use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CalendarEntrySource {
    Manual,
    Import,
    System,
}

impl Default for CalendarEntrySource {
    fn default() -> Self {
        Self::Manual
    }
}

impl CalendarEntrySource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Import => "import",
            Self::System => "system",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "manual" => Some(Self::Manual),
            "import" => Some(Self::Import),
            "system" => Some(Self::System),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CalendarEntry {
    pub event_id: uuid::Uuid,
    pub article_id: uuid::Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub location: Option<String>,
    pub description: Option<String>,
    pub dtstamp: Option<DateTime<Utc>>,
    pub source: CalendarEntrySource,
    pub created_by: Option<uuid::Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_by: Option<uuid::Uuid>,
    pub modified_at: Option<DateTime<Utc>>,
}
