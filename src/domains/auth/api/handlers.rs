use crate::{
    common::{
        app_state::AppState,
        dto::RestApiResponse,
        error::AppError,
        session_auth::{self, AuthSession, SessionUser},
    },
    domains::{
        auth::dto::auth_dto::{
            AuthPayload, AuthSessionDto, AuthUserDto, ResendVerificationEmailDto, SignUpDto,
            VerificationLinkDebugDto, VerifyEmailQueryDto,
        },
        notification::{NotificationKind, dto::notification_dto::EnqueueEmailNotificationDto},
        user::dto::user_dto::{CreateUserDto, UserDto},
    },
};
use axum::extract::{Query, State};
use axum::{Json, response::IntoResponse};
use validator::Validate;

#[utoipa::path(
    post,
    path = "/auth/signup",
    request_body = SignUpDto,
    responses((status = 200, description = "Create user and send email verification instructions", body = VerificationLinkDebugDto)),
    tag = "UserAuth"
)]
pub async fn sign_up_user(
    State(state): State<AppState>,
    Json(payload): Json<SignUpDto>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(AppError::from)?;

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
    let verification_url = enqueue_signup_verification_email(&state, &created_user).await?;

    Ok(RestApiResponse::success_with_message(
        "Bitte bestaetige zuerst deine E-Mail-Adresse. Wir haben dir einen Link geschickt.",
        VerificationLinkDebugDto {
            verification_url: maybe_expose_verification_url(&state, verification_url),
        },
    ))
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

#[utoipa::path(
    get,
    path = "/auth/verify-email",
    params(("token" = String, Query, description = "Email verification token")),
    responses((status = 200, description = "Verify email address")),
    tag = "UserAuth"
)]
pub async fn verify_email(
    State(state): State<AppState>,
    Query(query): Query<VerifyEmailQueryDto>,
) -> Result<impl IntoResponse, AppError> {
    state.auth_service.verify_email_token(query.token).await?;
    Ok(RestApiResponse::success_with_message(
        "E-Mail-Adresse bestaetigt.",
        (),
    ))
}

#[utoipa::path(
    post,
    path = "/auth/resend-verification",
    request_body = ResendVerificationEmailDto,
    responses((status = 200, description = "Resend email verification instructions", body = VerificationLinkDebugDto)),
    tag = "UserAuth"
)]
pub async fn resend_verification_email(
    State(state): State<AppState>,
    Json(payload): Json<ResendVerificationEmailDto>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(AppError::from)?;

    let email = payload.email.trim().to_string();
    let users = state
        .user_service
        .get_user_list(crate::domains::user::dto::user_dto::SearchUserDto {
            id: None,
            username: None,
            email: Some(email.clone()),
        })
        .await?;

    let mut verification_url = None;

    if let Some(user) = users.first()
        && user.email_verified_at.is_none()
    {
        verification_url = Some(enqueue_signup_verification_email(&state, user).await?);
    }

    Ok(RestApiResponse::success_with_message(
        "Wenn fuer diese E-Mail-Adresse ein unbestaetigtes Konto existiert, haben wir einen neuen Link geschickt.",
        VerificationLinkDebugDto {
            verification_url: verification_url
                .and_then(|url| maybe_expose_verification_url(&state, url)),
        },
    ))
}

fn roles_from_session_user(user: &SessionUser) -> Vec<String> {
    let mut roles: Vec<String> = user.roles.iter().cloned().collect();
    roles.sort();
    roles
}

async fn enqueue_signup_verification_email(
    state: &AppState,
    user: &UserDto,
) -> Result<String, AppError> {
    let verification_token = state
        .auth_service
        .issue_email_verification_token(user.id)
        .await?;
    let verification_url = format!(
        "{}/email-verifizieren?token={verification_token}",
        state.config.public_app_url.trim_end_matches('/')
    );
    let verification_subject = "Bitte bestaetige deine E-Mail-Adresse".to_string();
    let verification_body = format!(
        "Hallo {username},\n\nbitte bestaetige deine E-Mail-Adresse fuer Einerleih ueber diesen Link:\n{verification_url}\n\nDer Link ist 24 Stunden gueltig.",
        username = user.username
    );

    state
        .notification_service
        .enqueue_email(EnqueueEmailNotificationDto {
            kind: NotificationKind::SignupConfirmation,
            recipient_email: user.email.clone().unwrap_or_default(),
            subject: verification_subject,
            body_text: verification_body,
            booking_id: None,
            article_id: None,
            user_id: Some(user.id),
            created_by: Some(user.id),
        })
        .await?;

    if let Err(err) = state.notification_service.dispatch_pending(10).await {
        tracing::error!("Error dispatching signup verification email: {err}");
    }

    Ok(verification_url)
}

fn maybe_expose_verification_url(state: &AppState, verification_url: String) -> Option<String> {
    state
        .config
        .expose_email_verification_links
        .then_some(verification_url)
}
