use axum::http::{Method, StatusCode, header::SET_COOKIE};

use einerleih::{
    common::{config::Config, dto::RestApiResponse, session_auth},
    domains::auth::dto::auth_dto::{AuthPayload, AuthSessionDto},
};
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

    assert!(parts.headers.get(SET_COOKIE).is_some());

    let response_body: RestApiResponse<AuthSessionDto> = deserialize_json_body(body).await.unwrap();
    assert_eq!(response_body.0.status, StatusCode::OK);
    let session = response_body.0.data.unwrap();
    assert_eq!(session.user.username, TEST_CLIENT_ID);
    assert_eq!(session.roles, vec!["admin".to_string()]);
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

    assert!(parts.headers.get(SET_COOKIE).is_some());
    assert_eq!(session.user.username, payload["username"].as_str().unwrap());
    assert_eq!(session.user.email.as_deref(), payload["email"].as_str());
    assert_eq!(session.roles, vec!["user".to_string()]);
}

#[tokio::test]
async fn test_logout_user() {
    let session_cookie = test_helpers::login_and_get_session_cookie().await;
    let response =
        test_helpers::request_with_session(Method::POST, "/auth/logout", &session_cookie).await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);
    assert!(parts.headers.get(SET_COOKIE).is_some());

    let response_body: RestApiResponse<()> = deserialize_json_body(body).await.unwrap();
    assert_eq!(response_body.0.status, StatusCode::OK);

    let response =
        test_helpers::request_with_session(Method::GET, "/article/categories", &session_cookie)
            .await;
    let (parts, _) = response.into_parts();
    assert_eq!(parts.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_regular_user_cannot_access_admin_route() {
    let session_cookie = test_helpers::signup_and_get_session_cookie().await;
    let response = test_helpers::request_with_session(Method::GET, "/user", &session_cookie).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_bootstrap_admin_can_login_with_admin_role() {
    let pool = test_helpers::setup_test_db().await.unwrap();
    let mut config = Config::from_env().unwrap();
    let unique = uuid::Uuid::new_v4().simple().to_string();
    let username = format!("bootstrap-{unique}");
    let password = "temporary-admin-password";

    config.bootstrap_admin_username = Some(username.clone());
    config.bootstrap_admin_email = Some(format!("{username}@example.com"));
    config.bootstrap_admin_password = Some(password.to_string());

    session_auth::ensure_bootstrap_admin(&pool, &config)
        .await
        .unwrap();

    let payload = AuthPayload {
        client_id: username,
        client_secret: password.to_string(),
    };
    let response = request_with_body(Method::POST, "/auth/login", &payload).await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);
    let response_body: RestApiResponse<AuthSessionDto> = deserialize_json_body(body).await.unwrap();
    let session = response_body.0.data.unwrap();
    assert_eq!(session.roles, vec!["admin".to_string()]);
}
