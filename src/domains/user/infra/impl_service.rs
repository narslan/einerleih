use crate::{
    common::error::AppError,
    domains::user::{
        domain::{repository::UserRepository, service::UserServiceTrait},
        dto::user_dto::{CreateUserDto, SearchUserDto, UpdateUserDto, UserDto},
        infra::impl_repository::UserRepo,
    },
};
use async_trait::async_trait;
use deadpool_postgres::{Pool, PoolError};
use tokio_postgres::error::SqlState;

use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Service struct for handling user-related operations
/// such as creating, updating, deleting, and fetching users.
/// It uses a repository pattern to abstract the data access layer.
#[derive(Clone)]
pub struct UserService {
    pub pool: Pool,
    pub repo: Arc<dyn UserRepository + Send + Sync>,
}

fn field_validation_error(field: &str, message: &str) -> AppError {
    AppError::ValidationErrors {
        message: "Invalid input".into(),
        errors: HashMap::from([(field.to_string(), message.to_string())]),
    }
}

fn map_user_write_error(err: PoolError) -> AppError {
    if let PoolError::Backend(db_err) = &err {
        if let Some(db_error) = db_err.as_db_error()
            && db_error.code() == &SqlState::UNIQUE_VIOLATION
        {
            if db_error.constraint() == Some("users_username_key") {
                return field_validation_error(
                    "username",
                    "Dieser Benutzername ist bereits vergeben.",
                );
            }

            if db_error.constraint() == Some("users_email_key")
                || db_error.constraint() == Some("users_email_lower_key")
            {
                return field_validation_error(
                    "email",
                    "Diese E-Mail-Adresse wird bereits verwendet.",
                );
            }
        }
    }

    AppError::DatabaseError(err)
}

impl UserService {
    async fn ensure_email_is_available(
        &self,
        email: &str,
        exclude_user_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        match self.repo.find_by_email(self.pool.clone(), email).await {
            Ok(Some(user)) if Some(user.id) != exclude_user_id => Err(field_validation_error(
                "email",
                "Diese E-Mail-Adresse wird bereits verwendet.",
            )),
            Ok(_) => Ok(()),
            Err(err) => {
                tracing::error!("Error checking email uniqueness: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }
}

#[async_trait]
impl UserServiceTrait for UserService {
    /// constructor for the service.
    fn create_service(pool: Pool) -> Arc<dyn UserServiceTrait> {
        Arc::new(Self {
            pool,
            repo: Arc::new(UserRepo {}),
        })
    }

    /// Retrieves a user by their ID.
    async fn get_user_by_id(&self, id: Uuid) -> Result<UserDto, AppError> {
        match self.repo.find_by_id(self.pool.clone(), id).await {
            Ok(Some(user)) => Ok(UserDto::from(user)),
            Ok(None) => Err(AppError::NotFound("User not found".into())),
            Err(err) => {
                tracing::error!("Error retrieving user: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    /// Retrieves user list by condition
    /// Returns a vector of UserDto objects.
    async fn get_user_list(
        &self,
        search_user_dto: SearchUserDto,
    ) -> Result<Vec<UserDto>, AppError> {
        match self
            .repo
            .find_list(self.pool.clone(), search_user_dto)
            .await
        {
            Ok(users) => {
                let user_dtos: Vec<UserDto> = users.into_iter().map(Into::into).collect();
                Ok(user_dtos)
            }
            Err(err) => {
                tracing::error!("Error fetching users: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    /// Retrieves all users.
    /// Returns a vector of UserDto objects.
    async fn get_users(&self) -> Result<Vec<UserDto>, AppError> {
        match self.repo.find_all(self.pool.clone()).await {
            Ok(users) => {
                let user_dtos: Vec<UserDto> = users.into_iter().map(Into::into).collect();
                Ok(user_dtos)
            }
            Err(err) => {
                tracing::error!("Error fetching users: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }
    /// Creates a new user.
    async fn create_user(&self, create_user: CreateUserDto) -> Result<UserDto, AppError> {
        self.ensure_email_is_available(&create_user.email, None)
            .await?;

        let user_id = match self.repo.create(self.pool.clone(), create_user).await {
            Ok(user_id) => user_id,
            Err(err) => {
                tracing::error!("Error creating user: {err}");
                return Err(map_user_write_error(err));
            }
        };

        match self.repo.find_by_id(self.pool.clone(), user_id).await {
            Ok(Some(user)) => Ok(UserDto::from(user)),
            Ok(None) => Err(AppError::NotFound("User not found".into())),
            Err(err) => {
                tracing::error!("Error retrieving user: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    /// Updates an existing user.
    async fn update_user(&self, id: Uuid, payload: UpdateUserDto) -> Result<UserDto, AppError> {
        self.ensure_email_is_available(&payload.email, Some(id))
            .await?;

        match self.repo.update(self.pool.clone(), id, payload).await {
            Ok(Some(user)) => Ok(UserDto::from(user)),
            Ok(None) => Err(AppError::NotFound("User not found".into())),
            Err(err) => {
                tracing::error!("Error updating user: {err}");
                Err(map_user_write_error(err))
            }
        }
    }

    /// Deletes a user by their ID.
    async fn delete_user(&self, id: Uuid) -> Result<String, AppError> {
        match self.repo.delete(self.pool.clone(), id).await {
            Ok(true) => Ok("User deleted".into()),
            Ok(false) => Err(AppError::NotFound("User not found".into())),
            Err(err) => {
                tracing::error!("Error deleting user: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }
}
