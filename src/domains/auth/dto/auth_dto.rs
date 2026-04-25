use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

use crate::domains::user::dto::user_dto::UserDto;

static PASSWORD_MUST_CONTAIN_LETTER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[A-Za-z]").expect("password letter regex must compile"));
static PASSWORD_MUST_CONTAIN_DIGIT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\d").expect("password digit regex must compile"));

fn validate_password_complexity(password: &str) -> Result<(), ValidationError> {
    if !PASSWORD_MUST_CONTAIN_LETTER.is_match(password) {
        return Err(ValidationError::new("password_must_contain_letter")
            .with_message("Password must contain at least one letter".into()));
    }

    if !PASSWORD_MUST_CONTAIN_DIGIT.is_match(password) {
        return Err(ValidationError::new("password_must_contain_digit")
            .with_message("Password must contain at least one digit".into()));
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuthPayload {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct AuthUserDto {
    pub user_id: uuid::Uuid,
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct SignUpDto {
    #[validate(length(
        min = 3,
        max = 64,
        message = "Username must be between 3 and 64 characters"
    ))]
    pub username: String,
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    #[validate(custom(function = "validate_password_complexity"))]
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct ResendVerificationEmailDto {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthSessionDto {
    pub user: UserDto,
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct VerifyEmailQueryDto {
    pub token: String,
}
