use crate::{
    common::{
        app_state::AppState,
        dto::RestApiResponse,
        error::AppError,
        session_auth::{self, AuthSession, SessionUser},
    },
    domains::{
        auth::dto::auth_dto::{AuthPayload, AuthSessionDto, AuthUserDto, SignUpDto},
        user::dto::user_dto::CreateUserDto,
    },
};
use axum::extract::State;
use axum::{Json, response::IntoResponse};
use validator::Validate;

#[utoipa::path(
    post,
    path = "/auth/signup",
    request_body = SignUpDto,
    responses((status = 200, description = "Create user and login session", body = AuthSessionDto)),
    tag = "UserAuth"
)]
pub async fn sign_up_user(
    State(state): State<AppState>,
    mut auth_session: AuthSession,
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
    session_auth::assign_signup_role(&state.pool, created_user.id).await?;

    let auth_payload = AuthPayload {
        client_id: created_user.username.clone(),
        client_secret: payload.password,
    };
    let session_user = auth_session
        .authenticate(auth_payload)
        .await
        .map_err(|err| {
            tracing::error!("Error authenticating new user session: {err}");
            AppError::InternalError
        })?
        .ok_or(AppError::WrongCredentials)?;
    auth_session.login(&session_user).await.map_err(|err| {
        tracing::error!("Error creating login session after signup: {err}");
        AppError::InternalError
    })?;

    Ok(RestApiResponse::success(AuthSessionDto {
        user: created_user,
        roles: roles_from_session_user(&session_user),
    }))
}

/// this function creates a router for login user
#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = AuthPayload,
    responses((status = 200, description = "Login user", body = AuthSessionDto)),
    tag = "UserAuth"
)]
pub async fn login_user(
    State(state): State<AppState>,
    mut auth_session: AuthSession,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = state.auth_service.login_user(payload.clone()).await?;
    let session_user = auth_session
        .authenticate(payload)
        .await
        .map_err(|err| {
            tracing::error!("Error authenticating login session: {err}");
            AppError::InternalError
        })?
        .ok_or(AppError::WrongCredentials)?;
    auth_session.login(&session_user).await.map_err(|err| {
        tracing::error!("Error creating login session: {err}");
        AppError::InternalError
    })?;

    let user = state.user_service.get_user_by_id(user_id).await?;
    Ok(RestApiResponse::success(AuthSessionDto {
        user,
        roles: roles_from_session_user(&session_user),
    }))
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    responses((status = 200, description = "Logout current user")),
    tag = "UserAuth"
)]
pub async fn logout_user(mut auth_session: AuthSession) -> Result<impl IntoResponse, AppError> {
    auth_session.logout().await.map_err(|err| {
        tracing::error!("Error destroying login session: {err}");
        AppError::InternalError
    })?;

    Ok(RestApiResponse::success(()))
}

#[utoipa::path(
    get,
    path = "/auth/session",
    responses((status = 200, description = "Session status of user")),
    tag = "UserAuth"
)]
pub async fn session(
    State(state): State<AppState>,
    auth_session: AuthSession,
) -> Result<impl IntoResponse, AppError> {
    let Some(session_user) = auth_session.user else {
        return Err(AppError::Unauthorized);
    };

    let user = state.user_service.get_user_by_id(session_user.id).await?;

    Ok(RestApiResponse::success(AuthSessionDto {
        user,
        roles: roles_from_session_user(&session_user),
    }))
}

fn roles_from_session_user(user: &SessionUser) -> Vec<String> {
    let mut roles: Vec<String> = user.roles.iter().cloned().collect();
    roles.sort();
    roles
}
