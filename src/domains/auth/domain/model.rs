//! This module defines the `UserAuth` model used for representing
//! authentication data tied to a user.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a user's authentication information, including hashed password.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAuth {
    pub user_id: uuid::Uuid,
    pub password_hash: String,
    pub email_verified: bool,
}

#[derive(Debug, Clone)]
pub struct EmailVerificationToken {
    pub verification_token_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}
