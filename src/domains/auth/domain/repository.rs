use chrono::{DateTime, Utc};

use super::model::{EmailVerificationToken, UserAuth};

use async_trait::async_trait;
use deadpool_postgres::{Pool, PoolError, Transaction};
use uuid::Uuid;

#[async_trait]

pub trait UserAuthRepository: Send + Sync {
    /// Finds a user authentication record by the user's username.
    /// Returns `Ok(Some(UserAuth))` if found, or `Ok(None)` if not found.
    async fn find_by_user_name(
        &self,
        pool: Pool,
        user_name: String,
    ) -> Result<Option<UserAuth>, PoolError>;

    /// Inserts a new user authentication record into the database using a transaction.
    async fn create(&self, tx: &mut Transaction<'_>, user_auth: UserAuth) -> Result<(), PoolError>;

    async fn create_email_verification_token(
        &self,
        tx: &mut Transaction<'_>,
        token: EmailVerificationToken,
    ) -> Result<(), PoolError>;

    async fn find_active_email_verification_token(
        &self,
        pool: Pool,
        token: &str,
        now: DateTime<Utc>,
    ) -> Result<Option<EmailVerificationToken>, PoolError>;

    async fn consume_email_verification_token(
        &self,
        tx: &mut Transaction<'_>,
        verification_token_id: Uuid,
    ) -> Result<bool, PoolError>;

    async fn mark_email_verified(
        &self,
        tx: &mut Transaction<'_>,
        user_id: Uuid,
    ) -> Result<bool, PoolError>;
}
