use super::handlers::*;
use crate::{
    common::app_state::AppState,
    domains::article::dto::article_dto::{
        ArticleDto, ArticleRelationDto, CreateArticleDto, CreateArticleWithPicturesResponseDto,
        ExistingArticlePictureDto, NewArticlePictureDto, UpdateArticleDtoWithIdDto,
        UpdateArticleWithPicturesResponseDto,
    },
};

use axum::{
    Router,
    routing::{delete, get, post, put},
};

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_article_by_id,
        get_articles,
        get_my_articles,
        get_my_article_by_id,
        get_article_categories,
        get_article_towns,
        create_article,
        create_article_with_pictures,
        create_my_article_with_pictures,
        update_article,
        update_article_with_pictures,
        update_my_article_with_pictures,
        delete_article,
        delete_my_article,
    ),
    components(schemas(
        ArticleDto,
        ArticleRelationDto,
        CreateArticleDto,
        CreateArticleWithPicturesResponseDto,
        ExistingArticlePictureDto,
        NewArticlePictureDto,
        UpdateArticleDtoWithIdDto,
        UpdateArticleWithPicturesResponseDto
    )),
    tags(
        (name = "Articles", description = "Article management endpoints")
    )
)]
/// This struct is used to generate OpenAPI documentation for the article routes.
pub struct ArticleApiDoc;

pub fn public_article_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_articles))
        .route("/{id}", get(get_article_by_id))
}

pub fn protected_article_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_article))
        .route("/upload", post(create_article_with_pictures))
        .route("/{id}", put(update_article))
        .route("/{id}/upload", put(update_article_with_pictures))
        .route("/{id}", delete(delete_article))
}

pub fn user_article_routes() -> Router<AppState> {
    Router::new()
        .route("/categories", get(get_article_categories))
        .route("/towns", get(get_article_towns))
        .route("/mine", get(get_my_articles))
        .route("/mine/upload", post(create_my_article_with_pictures))
        .route("/mine/{id}", get(get_my_article_by_id))
        .route("/mine/{id}/upload", put(update_my_article_with_pictures))
        .route("/mine/{id}", delete(delete_my_article))
}
