//! This module defines the authentication service trait used to abstract
//! user login and registration logic.

use deadpool_postgres::Pool;
use std::sync::Arc;

use crate::{
    common::error::AppError,
    domains::auth::dto::auth_dto::{AuthPayload, AuthUserDto},
};

#[async_trait::async_trait]
/// Trait defining the contract for authentication-related operations.
/// Implementors are responsible for handling user creation and login logic.
pub trait AuthServiceTrait: Send + Sync {
    /// constructor for the service.
    fn create_service(pool: Pool) -> Arc<dyn AuthServiceTrait>
    where
        Self: Sized;

    /// Registers a new user authentication entry.
    async fn create_user_auth(&self, auth_user: AuthUserDto) -> Result<(), AppError>;

    /// Authenticates a user and returns the authenticated user id on success.
    async fn login_user(&self, auth_payload: AuthPayload) -> Result<uuid::Uuid, AppError>;

    /// Issues an email verification token for the given user and returns the raw token.
    async fn issue_email_verification_token(&self, user_id: uuid::Uuid)
    -> Result<String, AppError>;

    /// Verifies an email verification token and marks the user as verified.
    async fn verify_email_token(&self, token: String) -> Result<uuid::Uuid, AppError>;
}
