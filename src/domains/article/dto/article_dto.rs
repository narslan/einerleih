use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::domains::article::domain::model::{Article, ArticleStatus};
use crate::domains::file::{UploadedFile, dto::file_dto::UploadedFileDto};

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, ToSchema)]
pub struct ArticleRelationDto {
    pub id: uuid::Uuid,
    pub name: String,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, ToSchema)]
pub struct ArticleImageDto {
    pub id: uuid::Uuid,
    pub file_url: String,
    pub content_type: String,
    pub sort_order: i32,
    pub is_cover: bool,
}

#[derive(PartialEq, Debug, Deserialize, Serialize, ToSchema)]
pub struct ArticleDto {
    pub article_id: uuid::Uuid,
    pub name: String,
    pub category: ArticleRelationDto,
    pub description: String,
    pub town: ArticleRelationDto,
    pub status: ArticleStatus,
    pub cover_image: Option<ArticleImageDto>,
    pub pictures: Vec<ArticleImageDto>,

    pub created_by: Option<uuid::Uuid>,
    #[serde(with = "crate::common::ts_format::option")]
    pub created_at: Option<DateTime<Utc>>,
    pub modified_by: Option<uuid::Uuid>,
    #[serde(with = "crate::common::ts_format::option")]
    pub modified_at: Option<DateTime<Utc>>,
}


#[derive(PartialEq, Debug, Deserialize, serde::Serialize, ToSchema, Validate)]
pub struct CreateArticleDto {
    #[validate(length(max = 36, message = "Name cannot exceed 36 characters"))]
    pub name: String,
    pub category: uuid::Uuid,
    pub description: String,
    pub town: uuid::Uuid,
    pub status: ArticleStatus,
    #[serde(default)]
    pub created_by: uuid::Uuid,
    #[serde(default)]
    pub modified_by: uuid::Uuid,
}

/*#[derive(PartialEq, Debug, Deserialize, serde::Serialize, ToSchema)]
pub struct UpdateArticleDto {
    pub name: String,
    pub category: uuid::Uuid,
    pub description: String,
    pub town: uuid::Uuid,
    pub status: ArticleStatus,
    pub modified_by: uuid::Uuid,
}
*/

#[derive(PartialEq, Debug, Deserialize, serde::Serialize, ToSchema, Validate)]
pub struct UpdateArticleDtoWithIdDto {
    pub id: Option<uuid::Uuid>,
    #[validate(length(max = 36, message = "Name cannot exceed 36 characters"))]
    pub name: String,
    pub category: uuid::Uuid,
    pub description: String,
    pub town: uuid::Uuid,
    pub status: ArticleStatus,
    #[serde(default)]
    pub modified_by: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateArticleWithPicturesResponseDto {
    pub article: ArticleDto,
    pub uploaded_files: Vec<UploadedFileDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExistingArticlePictureDto {
    pub id: uuid::Uuid,
    pub sort_order: i32,
    pub is_cover: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewArticlePictureDto {
    pub sort_order: i32,
    pub is_cover: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateArticleWithPicturesResponseDto {
    pub article: ArticleDto,
    pub uploaded_files: Vec<UploadedFileDto>,
}

impl From<UploadedFile> for ArticleImageDto {
    fn from(value: UploadedFile) -> Self {
        Self {
            id: value.id,
            file_url: value.file_url,
            content_type: value.content_type,
            sort_order: value.sort_order,
            is_cover: value.is_cover,
        }
    }
}

impl ArticleDto {
    pub fn from_article_with_pictures(article: Article, pictures: Vec<UploadedFile>) -> Self {
        let mut picture_dtos: Vec<ArticleImageDto> =
            pictures.into_iter().map(ArticleImageDto::from).collect();
        picture_dtos.sort_by_key(|picture| (picture.sort_order, !picture.is_cover));
        let cover_image = picture_dtos.iter().find(|picture| picture.is_cover).cloned();

        Self {
            article_id: article.article_id,
            name: article.name,
            category: ArticleRelationDto {
                id: article.category_id,
                name: article.category_name,
            },
            description: article.description,
            town: ArticleRelationDto {
                id: article.town_id,
                name: article.town_name,
            },
            status: article.status,
            cover_image,
            pictures: picture_dtos,
            created_by: article.created_by,
            created_at: article.created_at,
            modified_by: article.modified_by,
            modified_at: article.modified_at,
        }
    }
}
