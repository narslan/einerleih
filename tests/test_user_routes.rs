use axum::http::{Method, StatusCode};

use einerleih::{
    common::{dto::RestApiResponse, error::AppError},
    domains::user::dto::user_dto::{CreateUserDto, UserDto},
};
mod test_helpers;

use test_helpers::{
    TEST_AUTH_USER_ID, deserialize_json_body, login_and_get_session_cookie,
    request_with_session_body,
};

async fn create_user() -> Result<(CreateUserDto, UserDto), AppError> {
    let username = format!("testuser-{}", uuid::Uuid::new_v4()).to_string();
    let email = format!("{}@test.com", username).to_string();
    let payload = CreateUserDto {
        username,
        email,
        modified_by: uuid::Uuid::new_v4(),
    };
    let session_cookie = login_and_get_session_cookie().await;
    let response = request_with_session_body(Method::POST, "/user", &session_cookie, &payload);
    let (parts, body) = response.await.into_parts();
    assert_eq!(parts.status, StatusCode::OK);
    let response_body: RestApiResponse<UserDto> = deserialize_json_body(body).await.unwrap();

    assert_eq!(response_body.0.status, StatusCode::OK);
    let user_dto = response_body.0.data.unwrap();

    Ok((payload, user_dto))
}

#[tokio::test]
async fn test_create_user() {
    let created = create_user().await.expect("Failed to create user");

    let payload = created.0;
    let user_dto = created.1;
    assert_ne!(user_dto.id, uuid::Uuid::nil());
    assert_eq!(user_dto.username, payload.username.clone());
    assert_eq!(user_dto.email, Some(payload.email.clone()));
    assert_eq!(
        user_dto.modified_by,
        Some(uuid::Uuid::parse_str(TEST_AUTH_USER_ID).unwrap())
    );
}
