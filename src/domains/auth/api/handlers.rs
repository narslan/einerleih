use crate::{
    common::{
        app_state::AppState,
        dto::RestApiResponse,
        error::AppError,
        jwt::{AuthBody, AuthPayload},
    },
    domains::{
        auth::dto::auth_dto::{AuthSessionDto, AuthUserDto, SignUpDto},
        user::dto::user_dto::CreateUserDto,
    },
};
use axum::extract::State;
use axum::{Json, response::IntoResponse};
use validator::Validate;

/// this function creates a router for creating user authentication registration
/// it will create a new user in the database
#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = AuthUserDto,
    responses((status = 200, description = "Create user authentication", body = AuthUserDto)),
    tag = "UserAuth"
)]
pub async fn create_user_auth(
    State(state): State<AppState>,
    Json(payload): Json<AuthUserDto>,
) -> Result<impl IntoResponse, AppError> {
    state.auth_service.create_user_auth(payload).await?;
    Ok(RestApiResponse::success(()))
}

#[utoipa::path(
    post,
    path = "/auth/signup",
    request_body = SignUpDto,
    responses((status = 200, description = "Create user and login session", body = AuthSessionDto)),
    tag = "UserAuth"
)]
pub async fn sign_up_user(
    State(state): State<AppState>,
    Json(payload): Json<SignUpDto>,
) -> Result<impl IntoResponse, AppError> {
    payload
        .validate()
        .map_err(|err| AppError::ValidationError(format!("Invalid input: {}", err)))?;

    let created_user = state
        .user_service
        .create_user(CreateUserDto {
            username: payload.username.clone(),
            email: payload.email.clone(),
            modified_by: uuid::Uuid::nil(),
        })
        .await?;

    state
        .auth_service
        .create_user_auth(AuthUserDto {
            user_id: created_user.id,
            password: payload.password.clone(),
        })
        .await?;

    let auth_body = state
        .auth_service
        .login_user(AuthPayload {
            client_id: created_user.username.clone(),
            client_secret: payload.password,
        })
        .await?;

    Ok(RestApiResponse::success(AuthSessionDto {
        user: created_user,
        token: auth_body.access_token,
    }))
}

/// this function creates a router for login user
/// it will return a JWT token if the user is authenticated
#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = AuthPayload,
    responses((status = 200, description = "Login user", body = AuthBody)),
    tag = "UserAuth"
)]
pub async fn login_user(
    State(state): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, AppError> {
    let auth_body = state.auth_service.login_user(payload).await?;
    Ok(RestApiResponse::success(auth_body))
}
