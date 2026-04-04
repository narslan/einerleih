use super::model::UserAuth;

use async_trait::async_trait;
use deadpool_postgres::{Pool, PoolError, Transaction};

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
}
