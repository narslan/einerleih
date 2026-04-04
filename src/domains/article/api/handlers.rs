use crate::{
    common::{
        app_state::AppState, dto::RestApiResponse, error::AppError, jwt::Claims,
        multipart_helper::{parse_multipart_to_maps, parse_multipart_to_multi_maps},
    },
    domains::article::{
        ArticleStatus,
        dto::article_dto::{
            ArticleDto, ArticleRelationDto, CreateArticleDto, CreateArticleWithPicturesResponseDto,
            ExistingArticlePictureDto, NewArticlePictureDto, UpdateArticleDtoWithIdDto,
            UpdateArticleWithPicturesResponseDto,
        },
    },
    domains::file::FileDto,
};

use axum::{Extension, Json, extract::{Multipart, State}, response::IntoResponse};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use validator::Validate;

static ARTICLE_PICTURE_ALLOWED_EXTENSIONS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\.(jpg|jpeg|png|webp)$").unwrap());

#[utoipa::path(
    get,
    path = "/article/{id}",
    responses((status = 200, description = "Get article by ID", body = ArticleDto)),
    tag = "Articles"
)]
pub async fn get_article_by_id(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let article = state.article_service.get_article_by_id(id).await?;
    Ok(RestApiResponse::success(article))
}

/*#[utoipa::path(
    post,
    path = "/article/list",
    request_body = SearchArticleDto,
    responses((status = 200, description = "List articles by condition", body = [ArticleDto])),
    tag = "Articles"
)]
pub async fn get_article_list(
    State(state): State<AppState>,
    Json(payload): Json<SearchArticleDto>,
) -> Result<impl IntoResponse, AppError> {
    let articles = state.article_service.get_article_list(payload).await?;
    Ok(RestApiResponse::success(articles))
}
*/
#[utoipa::path(
    get,
    path = "/article",
    responses((status = 200, description = "List all articles", body = [ArticleDto])),
    tag = "Articles"
)]
pub async fn get_articles(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let articles = state.article_service.get_articles().await?;
    Ok(RestApiResponse::success(articles))
}

#[utoipa::path(
    get,
    path = "/article/categories",
    responses((status = 200, description = "List article categories", body = [ArticleRelationDto])),
    tag = "Articles"
)]
pub async fn get_article_categories(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let categories = state.article_service.get_categories().await?;
    Ok(RestApiResponse::success(categories))
}

#[utoipa::path(
    get,
    path = "/article/towns",
    responses((status = 200, description = "List article towns", body = [ArticleRelationDto])),
    tag = "Articles"
)]
pub async fn get_article_towns(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let towns = state.article_service.get_towns().await?;
    Ok(RestApiResponse::success(towns))
}

#[utoipa::path(
    post,
    path = "/article",
    request_body = CreateArticleDto,
    responses((status = 200, description = "Create a new article", body = ArticleDto)),
    tag = "Articles"
)]
pub async fn create_article(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateArticleDto>,
) -> Result<impl IntoResponse, AppError> {
    let mut create_article = payload;
    create_article.created_by = claims.sub;
    create_article.modified_by = claims.sub;
    create_article
        .validate()
        .map_err(|err| AppError::ValidationError(format!("Invalid input: {}", err)))?;

    let article = state.article_service.create_article(create_article).await?;

    Ok(RestApiResponse::success(article))
}

#[utoipa::path(
    post,
    path = "/article/upload",
    responses((status = 200, description = "Create a new article with pictures", body = CreateArticleWithPicturesResponseDto)),
    tag = "Articles"
)]
pub async fn create_article_with_pictures(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let (fields, files): (
        std::collections::HashMap<String, String>,
        std::collections::HashMap<String, Vec<FileDto>>,
    ) =
        parse_multipart_to_maps(multipart, &ARTICLE_PICTURE_ALLOWED_EXTENSIONS).await?;

    let create_article = CreateArticleDto {
        name: required_field(&fields, "name")?,
        category: parse_uuid_field(&fields, "category")?,
        description: required_field(&fields, "description")?,
        town: parse_uuid_field(&fields, "town")?,
        status: parse_article_status(required_field(&fields, "status")?.as_str())?,
        created_by: claims.sub,
        modified_by: claims.sub,
    };

    create_article
        .validate()
        .map_err(|err| AppError::ValidationError(format!("Invalid input: {}", err)))?;

    let pictures = files.get("photos").cloned().unwrap_or_default();
    let new_picture_meta: Vec<NewArticlePictureDto> =
        parse_optional_json_field(&fields, "new_picture_meta")?.unwrap_or_default();
    let response = state
        .article_workflow_service
        .create_article_with_pictures(create_article, pictures, new_picture_meta)
        .await?;

    Ok(RestApiResponse::success(response))
}

#[utoipa::path(
    put,
    path = "/article/{id}",
    request_body = UpdateArticleDtoWithIdDto,
    responses((status = 200, description = "Update article", body = ArticleDto)),
    tag = "Articles"
)]
pub async fn update_article(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(payload): Json<UpdateArticleDtoWithIdDto>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(|err| {
        tracing::error!("Validation error: {err}");
        AppError::ValidationError(format!("Invalid input: {}", err))
    })?;

    // Set the modified_by field to the current article's ID.
    let mut payload = payload;
    payload.modified_by = claims.sub;

    let article = state.article_service.update_article(id, payload).await?;
    Ok(RestApiResponse::success(article))
}

#[utoipa::path(
    put,
    path = "/article/{id}/upload",
    responses((status = 200, description = "Update article with picture changes", body = UpdateArticleWithPicturesResponseDto)),
    tag = "Articles"
)]
pub async fn update_article_with_pictures(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let (fields, files) =
        parse_multipart_to_multi_maps(multipart, &ARTICLE_PICTURE_ALLOWED_EXTENSIONS).await?;

    let payload = UpdateArticleDtoWithIdDto {
        id: Some(id),
        name: required_multi_field(&fields, "name")?,
        category: parse_uuid_multi_field(&fields, "category")?,
        description: required_multi_field(&fields, "description")?,
        town: parse_uuid_multi_field(&fields, "town")?,
        status: parse_article_status(required_multi_field(&fields, "status")?.as_str())?,
        modified_by: claims.sub,
    };

    payload.validate().map_err(|err| {
        tracing::error!("Validation error: {err}");
        AppError::ValidationError(format!("Invalid input: {}", err))
    })?;

    let existing_pictures: Vec<ExistingArticlePictureDto> =
        parse_optional_json_multi_field(&fields, "existing_pictures")?.unwrap_or_default();
    let delete_file_ids: Vec<Uuid> =
        parse_optional_json_multi_field(&fields, "delete_file_ids")?.unwrap_or_default();
    let new_picture_meta: Vec<NewArticlePictureDto> =
        parse_optional_json_multi_field(&fields, "new_picture_meta")?.unwrap_or_default();
    let new_pictures = files.get("photos").cloned().unwrap_or_default();

    let response = state
        .article_workflow_service
        .update_article_with_pictures(
            id,
            payload,
            existing_pictures,
            delete_file_ids,
            new_pictures,
            new_picture_meta,
        )
        .await?;

    Ok(RestApiResponse::success(response))
}

#[utoipa::path(
    delete,
    path = "/article/{id}",
    responses((status = 200, description = "Article deleted")),
    tag = "Articles"
)]
pub async fn delete_article(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let message = state.article_service.delete_article(id).await?;
    Ok(RestApiResponse::success_with_message(message, ()))
}

fn required_field(
    fields: &std::collections::HashMap<String, String>,
    key: &str,
) -> Result<String, AppError> {
    fields
        .get(key)
        .cloned()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError::ValidationError(format!("Missing field: {key}")))
}

fn parse_uuid_field(
    fields: &std::collections::HashMap<String, String>,
    key: &str,
) -> Result<Uuid, AppError> {
    let value = required_field(fields, key)?;
    Uuid::parse_str(&value)
        .map_err(|_| AppError::ValidationError(format!("Invalid UUID for field: {key}")))
}

fn parse_article_status(value: &str) -> Result<ArticleStatus, AppError> {
    ArticleStatus::from_db(value)
        .ok_or_else(|| AppError::ValidationError(format!("Invalid article status: {value}")))
}

fn parse_optional_json_field<T: DeserializeOwned>(
    fields: &std::collections::HashMap<String, String>,
    key: &str,
) -> Result<Option<T>, AppError> {
    let Some(value) = fields.get(key) else {
        return Ok(None);
    };

    if value.trim().is_empty() {
        return Ok(None);
    }

    serde_json::from_str(value).map(Some).map_err(|err| {
        AppError::ValidationError(format!("Invalid JSON for field {key}: {err}"))
    })
}

fn required_multi_field(
    fields: &std::collections::HashMap<String, Vec<String>>,
    key: &str,
) -> Result<String, AppError> {
    fields
        .get(key)
        .and_then(|values| values.last())
        .cloned()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError::ValidationError(format!("Missing field: {key}")))
}

fn parse_uuid_multi_field(
    fields: &std::collections::HashMap<String, Vec<String>>,
    key: &str,
) -> Result<Uuid, AppError> {
    let value = required_multi_field(fields, key)?;
    Uuid::parse_str(&value)
        .map_err(|_| AppError::ValidationError(format!("Invalid UUID for field: {key}")))
}

fn parse_optional_json_multi_field<T: DeserializeOwned>(
    fields: &std::collections::HashMap<String, Vec<String>>,
    key: &str,
) -> Result<Option<T>, AppError> {
    let Some(value) = fields.get(key).and_then(|values| values.last()) else {
        return Ok(None);
    };

    if value.trim().is_empty() {
        return Ok(None);
    }

    serde_json::from_str(value).map(Some).map_err(|err| {
        AppError::ValidationError(format!("Invalid JSON for field {key}: {err}"))
    })
}
