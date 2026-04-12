use super::handlers::*;
use crate::{common::app_state::AppState, domains::file::dto::file_dto::UploadedFileDto};
use axum::{
    Router,
    routing::{delete, get},
};

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        serve_protected_file,
        delete_file,
    ),
    components(schemas(UploadedFileDto)),
    tags(
        (name = "Files", description = "File management endpoints")
    )
)]
/// FileApiDoc is used to generate OpenAPI documentation for the file API.
pub struct FileApiDoc;

pub fn public_file_routes() -> Router<AppState> {
    Router::new().route("/{file_id}", get(serve_protected_file))
}

pub fn protected_file_routes() -> Router<AppState> {
    Router::new().route("/{file_id}", delete(delete_file))
}
