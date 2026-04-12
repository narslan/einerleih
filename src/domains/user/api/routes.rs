use super::handlers::*;
use crate::{
    common::app_state::AppState,
    domains::user::dto::user_dto::{CreateUserDto, SearchUserDto, UpdateUserDto, UserDto},
};

use axum::{
    Router,
    routing::{delete, get, post, put},
};

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_user_by_id,
        get_users,
        get_user_list,
        create_user,
        update_user,
        delete_user,
    ),
    components(schemas(UserDto, SearchUserDto, CreateUserDto, UpdateUserDto)),
    tags(
        (name = "Users", description = "User management endpoints")
    )
)]
/// This struct is used to generate OpenAPI documentation for the user routes.
pub struct UserApiDoc;

pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_users))
        .route("/", post(create_user))
        .route("/list", post(get_user_list))
        .route("/{id}", get(get_user_by_id))
        .route("/{id}", put(update_user))
        .route("/{id}", delete(delete_user))
}
