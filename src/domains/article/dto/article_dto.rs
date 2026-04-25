use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

use crate::domains::article::domain::model::{Article, ArticleStatus};
use crate::domains::file::{UploadedFile, dto::file_dto::UploadedFileDto};

const MAX_TAG_COUNT: usize = 12;
const MAX_TAG_LENGTH: usize = 64;

fn validate_article_description(description: &str) -> Result<(), ValidationError> {
    if description.trim().is_empty() {
        return Err(ValidationError::new("article_description_required")
            .with_message("Description must not be empty".into()));
    }

    Ok(())
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, ToSchema)]
pub struct ArticleRelationDto {
    pub id: uuid::Uuid,
    pub name: String,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, ToSchema)]
pub struct ArticleTagDto {
    pub id: uuid::Uuid,
    pub slug: String,
    pub name: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct NormalizedArticleTag {
    pub slug: String,
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
    pub tags: Vec<ArticleTagDto>,

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
    #[validate(length(max = 1000, message = "Description cannot exceed 1000 characters"))]
    #[validate(custom(function = "validate_article_description"))]
    pub description: String,
    pub town: uuid::Uuid,
    pub status: ArticleStatus,
    #[serde(default)]
    pub tags: Vec<String>,
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
    #[validate(length(max = 1000, message = "Description cannot exceed 1000 characters"))]
    #[validate(custom(function = "validate_article_description"))]
    pub description: String,
    pub town: uuid::Uuid,
    pub status: ArticleStatus,
    #[serde(default)]
    pub tags: Vec<String>,
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
    pub fn from_article_with_pictures(
        article: Article,
        pictures: Vec<UploadedFile>,
        tags: Vec<ArticleTagDto>,
    ) -> Self {
        let mut picture_dtos: Vec<ArticleImageDto> =
            pictures.into_iter().map(ArticleImageDto::from).collect();
        picture_dtos.sort_by_key(|picture| (picture.sort_order, !picture.is_cover));
        let cover_image = picture_dtos
            .iter()
            .find(|picture| picture.is_cover)
            .cloned();

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
            tags,
            created_by: article.created_by,
            created_at: article.created_at,
            modified_by: article.modified_by,
            modified_at: article.modified_at,
        }
    }
}

pub fn normalize_article_tags(raw_tags: &[String]) -> Result<Vec<NormalizedArticleTag>, String> {
    let mut tags = Vec::new();

    for raw_tag in raw_tags {
        for candidate in raw_tag.split(',') {
            let Some(tag) = normalize_article_tag(candidate)? else {
                continue;
            };

            if tags
                .iter()
                .any(|existing: &NormalizedArticleTag| existing.slug == tag.slug)
            {
                continue;
            }

            tags.push(tag);

            if tags.len() > MAX_TAG_COUNT {
                return Err(format!("Maximal {MAX_TAG_COUNT} Tags sind erlaubt"));
            }
        }
    }

    Ok(tags)
}

fn normalize_article_tag(raw_tag: &str) -> Result<Option<NormalizedArticleTag>, String> {
    let name = raw_tag
        .trim()
        .trim_start_matches('#')
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if name.is_empty() {
        return Ok(None);
    }

    if name.chars().count() > MAX_TAG_LENGTH {
        return Err(format!(
            "Ein Tag darf maximal {MAX_TAG_LENGTH} Zeichen lang sein"
        ));
    }

    let mut slug = String::new();
    let mut last_was_separator = false;

    for character in name.to_lowercase().chars() {
        if character.is_alphanumeric() {
            slug.push(character);
            last_was_separator = false;
        } else if character.is_whitespace() || character == '-' || character == '_' {
            if !slug.is_empty() && !last_was_separator {
                slug.push('-');
                last_was_separator = true;
            }
        }
    }

    let slug = slug.trim_matches('-').to_string();

    if slug.is_empty() {
        return Err(format!("Ungültiger Tag: {raw_tag}"));
    }

    Ok(Some(NormalizedArticleTag { slug, name }))
}
