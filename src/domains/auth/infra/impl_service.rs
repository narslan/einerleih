use crate::{
    common::{error::AppError, hash_utils},
    domains::auth::{
        domain::{
            model::{EmailVerificationToken, UserAuth},
            repository::UserAuthRepository,
            service::AuthServiceTrait,
        },
        dto::auth_dto::{AuthPayload, AuthUserDto},
        infra::impl_repository::UserAuthRepo,
    },
};
use chrono::{Duration, Utc};

use deadpool_postgres::Pool;

use std::sync::Arc;

/// Service for handling user authentication
/// and authorization logic.
#[derive(Clone)]
pub struct AuthService {
    pool: Pool,
    repo: Arc<dyn UserAuthRepository + Send + Sync>,
}

/// Implementation of the AuthService
#[async_trait::async_trait]
impl AuthServiceTrait for AuthService {
    /// constructor for the service.
    fn create_service(pool: Pool) -> Arc<dyn AuthServiceTrait> {
        Arc::new(Self {
            pool,
            repo: Arc::new(UserAuthRepo {}),
        })
    }

    /// It hashes the password and stores it in the database.
    async fn create_user_auth(&self, auth_user: AuthUserDto) -> Result<(), AppError> {
        let mut client = self.pool.get().await.unwrap();
        let mut tx = client.transaction().await.unwrap();

        let password_hash =
            hash_utils::hash_password(&auth_user.password).map_err(|_| AppError::InternalError)?;

        let user_auth = UserAuth {
            user_id: auth_user.user_id,
            password_hash,
            email_verified: false,
        };

        match self.repo.create(&mut tx, user_auth).await {
            Ok(()) => {
                tx.commit().await.unwrap();
                Ok(())
            }
            Err(err) => {
                tracing::error!("Error creating user auth: {err}");
                tx.rollback().await.unwrap();
                Err(AppError::DatabaseError(err))
            }
        }
    }

    /// Authenticates a user by checking the provided credentials
    /// against the stored credentials in the database.
    /// If the credentials are invalid, it returns an error.
    async fn login_user(&self, auth_payload: AuthPayload) -> Result<uuid::Uuid, AppError> {
        if auth_payload.client_id.is_empty() || auth_payload.client_secret.is_empty() {
            return Err(AppError::MissingCredentials);
        }

        let user_auth = self
            .repo
            .find_by_user_name(self.pool.clone(), auth_payload.client_id.clone())
            .await
            .map_err(AppError::DatabaseError)?;

        let user_auth = user_auth.ok_or(AppError::UserNotFound)?;

        if !user_auth.email_verified {
            return Err(AppError::ValidationError(
                "Bitte bestaetige zuerst deine E-Mail-Adresse.".into(),
            ));
        }

        if !hash_utils::verify_password(&user_auth.password_hash, &auth_payload.client_secret) {
            return Err(AppError::WrongCredentials);
        }

        Ok(user_auth.user_id)
    }

    async fn issue_email_verification_token(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<String, AppError> {
        let raw_token = format!(
            "{}{}",
            uuid::Uuid::new_v4().simple(),
            uuid::Uuid::new_v4().simple()
        );
        let token = EmailVerificationToken {
            verification_token_id: uuid::Uuid::new_v4(),
            user_id,
            token: raw_token.clone(),
            expires_at: Utc::now() + Duration::hours(24),
        };

        let mut client = self.pool.get().await.map_err(AppError::DatabaseError)?;
        let mut tx = client.transaction().await.map_err(|err| {
            tracing::error!("Error starting email verification transaction: {err}");
            AppError::InternalError
        })?;

        match self
            .repo
            .create_email_verification_token(&mut tx, token)
            .await
        {
            Ok(()) => {
                tx.commit().await.map_err(|err| {
                    tracing::error!("Error committing email verification transaction: {err}");
                    AppError::InternalError
                })?;
                Ok(raw_token)
            }
            Err(err) => {
                tracing::error!("Error creating email verification token: {err}");
                let _ = tx.rollback().await;
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn verify_email_token(&self, token: String) -> Result<uuid::Uuid, AppError> {
        let token = token.trim().to_string();
        if token.is_empty() {
            return Err(AppError::ValidationError("token is required".into()));
        }

        let verification = self
            .repo
            .find_active_email_verification_token(self.pool.clone(), &token, Utc::now())
            .await
            .map_err(AppError::DatabaseError)?
            .ok_or_else(|| {
                AppError::ValidationError("Invalid or expired verification token".into())
            })?;

        let mut client = self.pool.get().await.map_err(AppError::DatabaseError)?;
        let mut tx = client.transaction().await.map_err(|err| {
            tracing::error!("Error starting email verification consume transaction: {err}");
            AppError::InternalError
        })?;

        let marked = self
            .repo
            .mark_email_verified(&mut tx, verification.user_id)
            .await
            .map_err(AppError::DatabaseError)?;
        if !marked {
            let _ = tx.rollback().await;
            return Err(AppError::UserNotFound);
        }

        self.repo
            .consume_email_verification_token(&mut tx, verification.verification_token_id)
            .await
            .map_err(AppError::DatabaseError)?;

        tx.commit().await.map_err(|err| {
            tracing::error!("Error committing email verification consume transaction: {err}");
            AppError::InternalError
        })?;

        Ok(verification.user_id)
    }
}
