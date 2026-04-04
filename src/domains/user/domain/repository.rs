//! This module defines the `UserRepository` trait, which abstracts
//! the database operations related to user entities.

use crate::domains::user::dto::user_dto::{CreateUserDto, SearchUserDto, UpdateUserDto};

use super::model::User;

use async_trait::async_trait;
use deadpool_postgres::{Pool, PoolError};
use uuid::Uuid;

#[async_trait]
/// Trait representing repository-level operations for user entities.
/// Provides methods for creating, retrieving, updating, and deleting users in the database.
pub trait UserRepository: Send + Sync {
    /// Retrieves all users from the database.
    async fn find_all(&self, pool: Pool) -> Result<Vec<User>, PoolError>;

    /// Finds a user by their unique identifier.
    async fn find_by_id(&self, pool: Pool, id: Uuid) -> Result<Option<User>, PoolError>;

    /// Finds user list by condition
    async fn find_list(
        &self,
        pool: Pool,
        search_user_dto: SearchUserDto,
    ) -> Result<Vec<User>, PoolError>;

    /// Creates a new user record using the provided data within an active transaction.
    async fn create(&self, pool: Pool, user: CreateUserDto) -> Result<Uuid, PoolError>;

    /// Updates an existing user record using the provided data.
    async fn update(
        &self,
        pool: Pool,
        id: Uuid,
        user: UpdateUserDto,
    ) -> Result<Option<User>, PoolError>;

    /// Deletes a user by their unique identifier within an active transaction.
    async fn delete(&self, pool: Pool, id: Uuid) -> Result<bool, PoolError>;
}
