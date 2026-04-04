use chrono::{DateTime, Utc};
/// Domain model representing a user in the application.
#[derive(Debug, Clone)]
pub struct User {
    pub id: uuid::Uuid,
    pub username: String,
    pub email: Option<String>,
    pub created_by: Option<uuid::Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_by: Option<uuid::Uuid>,
    pub modified_at: Option<DateTime<Utc>>,
}
