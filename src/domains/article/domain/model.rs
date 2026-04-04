//! Domain model definitions for article-related entities.
//! This includes enums for article status and OS, as well as the core `Article` struct.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Enum representing the possible statuses of an article.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ArticleStatus {
    Aktiv,
    Ausgedient,
}

impl ArticleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Aktiv => "aktiv",
            Self::Ausgedient => "ausgedient",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "aktiv" => Some(Self::Aktiv),
            "ausgedient" => Some(Self::Ausgedient),
            _ => None,
        }
    }
}

/// Domain model representing an article entity.
#[derive(Debug, Clone)]
pub struct Article {
    pub article_id: uuid::Uuid,
    pub name: String,
    pub category_id: uuid::Uuid,
    pub category_name: String,
    pub description: String,
    pub town_id: uuid::Uuid,
    pub town_name: String,
    pub status: ArticleStatus,
    pub created_by: Option<uuid::Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_by: Option<uuid::Uuid>,
    pub modified_at: Option<DateTime<Utc>>,
}
