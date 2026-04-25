use crate::common::app_state::AppState;
use axum::{
    Router,
    routing::{get, post},
};

use super::handlers;

use utoipa::OpenApi;

/// Import the necessary modules for OpenAPI documentation generation
#[derive(OpenApi)]
#[openapi(
    paths(
        super::handlers::login_user,
        super::handlers::logout_user,
        super::handlers::sign_up_user,
        super::handlers::session,
        super::handlers::verify_email,
        super::handlers::resend_verification_email,
    ),
    components(schemas(
        crate::domains::auth::dto::auth_dto::AuthPayload,
        crate::domains::auth::dto::auth_dto::SignUpDto,
        crate::domains::auth::dto::auth_dto::ResendVerificationEmailDto,
        crate::domains::auth::dto::auth_dto::AuthSessionDto,
        crate::domains::auth::dto::auth_dto::VerifyEmailQueryDto,
    )),
    tags(
        (name = "UserAuth", description = "User authentication endpoints")
    )
)]
/// This struct is used to generate OpenAPI documentation for the user authentication routes.
pub struct UserAuthApiDoc;

/// This function creates a router for the user authentication routes.
/// It defines the routes and their corresponding handlers.
pub fn user_auth_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(handlers::login_user))
        .route("/logout", post(handlers::logout_user))
        .route("/signup", post(handlers::sign_up_user))
        .route("/session", get(handlers::session))
        .route("/verify-email", get(handlers::verify_email))
        .route(
            "/resend-verification",
            post(handlers::resend_verification_email),
        )
}
