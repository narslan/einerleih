use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use simple_dto_mapper_derive::DtoFrom;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domains::file::domain::model::{UploadedFile, UploadedFileType};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
pub struct FileDto {
    pub content_type: String,
    pub original_filename: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateFileDto {
    pub id: Uuid,
    pub file_name: String,
    pub origin_file_name: String,
    pub file_relative_path: String,
    pub file_url: String,
    pub content_type: String,
    pub file_size: i64,
    pub file_type: UploadedFileType,
    pub article_id: Uuid,
    pub sort_order: i32,
    pub is_cover: bool,

    pub modified_by: Uuid,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UploadFileDto {
    pub file: FileDto,
    pub article_id: Uuid,
    pub modified_by: Uuid,
    pub sort_order: i32,
    pub is_cover: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, DtoFrom)]
#[dto(from = UploadedFile)]
pub struct UploadedFileDto {
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
    #[serde(with = "crate::common::ts_format::option")]
    pub created_at: Option<DateTime<Utc>>,
    pub modified_by: Option<Uuid>,
    #[serde(with = "crate::common::ts_format::option")]
    pub modified_at: Option<DateTime<Utc>>,
}
