use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use simple_dto_mapper_derive::DtoFrom;
use utoipa::ToSchema;
use validator::Validate;

use crate::domains::user::domain::model::User;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, DtoFrom)]
#[dto(from = User)]
pub struct UserDto {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub created_by: Option<uuid::Uuid>,
    #[serde(with = "crate::common::ts_format::option")]
    pub created_at: Option<DateTime<Utc>>,
    pub modified_by: Option<uuid::Uuid>,
    #[serde(with = "crate::common::ts_format::option")]
    pub modified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchUserDto {
    pub id: Option<Uuid>,
    pub username: Option<String>,
    pub email: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateUserDto {
    #[validate(length(
        min = 3,
        max = 64,
        message = "Username must be between 3 and 64 characters"
    ))]
    pub username: String,
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[serde(default)]
    pub modified_by: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateUserDto {
    #[validate(length(
        min = 3,
        max = 64,
        message = "Username must be between 3 and 64 characters"
    ))]
    pub username: String,
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[serde(default)]
    pub modified_by: uuid::Uuid,
}
