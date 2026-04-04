use axum::http::{Method, StatusCode};

use einerleih::common::{
    dto::RestApiResponse,
    jwt::{AuthBody, AuthPayload},
};
use einerleih::domains::auth::dto::auth_dto::AuthSessionDto;
use test_helpers::{TEST_CLIENT_ID, TEST_CLIENT_SECRET, deserialize_json_body, request_with_body};

mod test_helpers;

#[tokio::test]
async fn test_login_user() {
    let payload = AuthPayload {
        client_id: TEST_CLIENT_ID.to_string(),
        client_secret: TEST_CLIENT_SECRET.to_string(),
    };

    let response = request_with_body(Method::POST, "/auth/login", &payload);

    let (parts, body) = response.await.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<AuthBody> = deserialize_json_body(body).await.unwrap();

    assert_eq!(response_body.0.status, StatusCode::OK);

    let auth_body = response_body.0.data.unwrap();

    assert_eq!(auth_body.token_type, "Bearer");
    assert!(!auth_body.access_token.is_empty());
}

#[tokio::test]
async fn test_login_user_fail() {
    let payload = AuthPayload {
        client_id: TEST_CLIENT_ID.to_string(),
        client_secret: uuid::Uuid::new_v4().to_string(),
    };

    let response = request_with_body(Method::POST, "/auth/login", &payload);

    let (parts, body) = response.await.into_parts();

    assert_eq!(parts.status, StatusCode::UNAUTHORIZED);

    let response_body: RestApiResponse<()> = deserialize_json_body(body).await.unwrap();

    assert_eq!(response_body.0.status, StatusCode::UNAUTHORIZED);
    // println!("response_body.0.status: {:?}", response_body.0.status);
    // println!("response_body.0.message: {:?}", response_body.0.message);
}

#[tokio::test]
async fn test_login_user_not_found() {
    let username = format!("testuser-{}", uuid::Uuid::new_v4()).to_string();

    let payload = AuthPayload {
        client_id: username,
        client_secret: uuid::Uuid::new_v4().to_string(),
    };

    let response = request_with_body(Method::POST, "/auth/login", &payload);

    let (parts, body) = response.await.into_parts();

    assert_eq!(parts.status, StatusCode::NOT_FOUND);

    let response_body: RestApiResponse<()> = deserialize_json_body(body).await.unwrap();

    assert_eq!(response_body.0.status, StatusCode::NOT_FOUND);
    println!("response_body.0.status: {:?}", response_body.0.status);
    println!("response_body.0.message: {:?}", response_body.0.message);
}

#[tokio::test]
async fn test_signup_user() {
    let unique = uuid::Uuid::new_v4().simple().to_string();
    let payload = serde_json::json!({
        "username": format!("user-{unique}"),
        "email": format!("user-{unique}@example.com"),
        "password": "test_password"
    });

    let response = request_with_body(Method::POST, "/auth/signup", &payload).await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<AuthSessionDto> = deserialize_json_body(body).await.unwrap();
    let session = response_body.0.data.unwrap();

    assert_eq!(session.user.username, payload["username"].as_str().unwrap());
    assert_eq!(session.user.email.as_deref(), payload["email"].as_str());
    assert!(!session.token.is_empty());
}
