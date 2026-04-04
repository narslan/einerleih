//! Domain model definitions related to uploaded files.
//! This includes the `FileType` enum and `UploadedFile` struct,
//! used to represent file metadata in the business logic layer.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;
use uuid::Uuid;

/// Enum representing different categories of files stored in the system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UploadedFileType {
    Foto,
    Dokument,
    Andere,
}

impl UploadedFileType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Foto => "foto",
            Self::Dokument => "dokument",
            Self::Andere => "andere",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "foto" => Some(Self::Foto),
            "dokument" => Some(Self::Dokument),
            "andere" => Some(Self::Andere),
            _ => None,
        }
    }
}

impl fmt::Display for UploadedFileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Domain model representing metadata for a file uploaded by admin.
#[derive(Debug, Clone)]
pub struct UploadedFile {
    pub id: Uuid,
    pub file_name: String,
    pub origin_file_name: String,
    pub file_relative_path: String,
    pub file_url: String,
    pub content_type: String,
    pub file_size: i64,
    pub file_type: UploadedFileType,
    pub article_id: Option<Uuid>,
    pub sort_order: i32,
    pub is_cover: bool,

    pub created_by: Option<Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_by: Option<Uuid>,
    pub modified_at: Option<DateTime<Utc>>,
}
