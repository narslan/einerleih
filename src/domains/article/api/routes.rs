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

use utoipa::{
    OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        get_article_by_id,
        get_articles,
        get_article_categories,
        get_article_towns,
        create_article,
        create_article_with_pictures,
        update_article,
        update_article_with_pictures,
        delete_article,
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
    ),
    security(
        ("bearer_auth" = [])
    ),
    modifiers(&ArticleApiDoc)
)]
/// This struct is used to generate OpenAPI documentation for the article routes.
pub struct ArticleApiDoc;

impl utoipa::Modify for ArticleApiDoc {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some("Input your `<your‑jwt>`"))
                    .build(),
            ),
        )
    }
}

pub fn public_article_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_articles))
        .route("/{id}", get(get_article_by_id))
}

pub fn protected_article_routes() -> Router<AppState> {
    Router::new()
        .route("/categories", get(get_article_categories))
        .route("/towns", get(get_article_towns))
        .route("/", post(create_article))
        .route("/upload", post(create_article_with_pictures))
        .route("/{id}", put(update_article))
        .route("/{id}/upload", put(update_article_with_pictures))
        .route("/{id}", delete(delete_article))
}
