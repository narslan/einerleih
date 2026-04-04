use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::domains::user::dto::user_dto::UserDto;

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct AuthUserDto {
    pub user_id: uuid::Uuid,
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct SignUpDto {
    #[validate(length(max = 64, message = "Username cannot exceed 64 characters"))]
    pub username: String,
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthSessionDto {
    pub user: UserDto,
    pub token: String,
}
