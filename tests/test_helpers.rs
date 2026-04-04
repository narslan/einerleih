use std::sync::{LazyLock, Once};

use axum::{
    Router,
    body::Body,
    http::{
        Method, Request, Response,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
};
use http_body_util::BodyExt;

use deadpool_postgres::{Pool, Runtime};
use einerleih::{
    app::create_router,
    common::{
        bootstrap::build_app_state,
        config::Config,
        dto::RestApiResponse,
        hash_utils,
        jwt::{AuthBody, AuthPayload},
    },
};

use dotenvy::from_filename;
use tokio::sync::Mutex;
use tower::ServiceExt;
use uuid::Uuid;

type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Constants for test client credentials
/// These are used to authenticate the test client
#[allow(dead_code)]
pub const TEST_CLIENT_ID: &str = "apitest01";
#[allow(dead_code)]
pub const TEST_CLIENT_SECRET: &str = "test_password";
#[allow(dead_code)]
pub const TEST_AUTH_USER_ID: &str = "00000000-0000-0000-0000-000000000021";

#[allow(dead_code)]
pub const TEST_USER_ID: &str = "00000000-0000-0000-0000-000000000001";
#[allow(dead_code)]
pub const TEST_TOWN_ID: &str = "00000000-0000-0000-0000-000000000101";
#[allow(dead_code)]
pub const TEST_CATEGORY_ID: &str = "00000000-0000-0000-0000-000000000201";

static INIT: Once = Once::new();
static TEST_DB_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
static TEST_DB_PREPARED: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));

/// Helper function to load environment variables from .env.test file
fn load_test_env() {
    INIT.call_once(|| {
        from_filename(".env.test").expect("Failed to load .env.test");
    });
}

/// Helper function to set up the test database state
pub async fn setup_test_db() -> TestResult<Pool> {
    load_test_env();
    let config = Config::from_env()?;
    let pool = config
        .pg
        .create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)
        .unwrap();
    {
        let _guard = TEST_DB_LOCK.lock().await;
        let mut prepared = TEST_DB_PREPARED.lock().await;
        if !*prepared {
            einerleih::common::db_migrations::reset_schema(&pool).await?;
            ensure_test_fixtures(&pool).await?;
            *prepared = true;
        }
    }
    Ok(pool)
}

/// Helper function to create a test router
pub async fn create_test_router() -> Router {
    let pool = setup_test_db().await.unwrap();
    let config = Config::from_env().unwrap();
    let state = build_app_state(pool, config.clone());
    let app = create_router(state);

    app
}

/// Helper function to deserialize the body of a request into a specific type
pub async fn deserialize_json_body<T: serde::de::DeserializeOwned>(
    body: Body,
) -> TestResult<T> {
    let bytes = body
        .collect()
        .await
        .map_err(|e| {
            tracing::error!("Failed to collect response body: {}", e);
            e
        })?
        .to_bytes();

    if bytes.is_empty() {
        return Err(("Empty response body").into());
    }

    // Debugging output
    // Uncomment the following lines to print the response body
    // if let Ok(body) = std::str::from_utf8(&bytes) {
    //     println!("body = {body:?}");
    // }

    let parsed = serde_json::from_slice::<T>(&bytes)?;

    Ok(parsed)
}

/// Helper functions to create a request
#[allow(dead_code)]
pub async fn request(method: Method, uri: &str) -> Response<Body> {
    let request = get_request(method, uri);
    let app = create_test_router().await;

    app.oneshot(request.await).await.unwrap()
}

/// Helper function to create a request with a body
#[allow(dead_code)]
pub async fn request_with_body<T: serde::Serialize>(
    method: Method,
    uri: &str,
    payload: &T,
) -> Response<Body> {
    let json_payload = serde_json::to_string(payload).expect("Failed to serialize payload");
    let request = get_request_with_body(method, uri, &json_payload);
    let app = create_test_router().await;

    app.oneshot(request.await).await.unwrap()
}

#[allow(dead_code)]
pub async fn request_with_auth_body<T: serde::Serialize>(
    method: Method,
    uri: &str,
    token: &str,
    payload: &T,
) -> Response<Body> {
    let json_payload = serde_json::to_string(payload).expect("Failed to serialize payload");
    let request = get_request_with_auth_body(method, uri, token, &json_payload);
    let app = create_test_router().await;

    app.oneshot(request.await).await.unwrap()
}

#[allow(dead_code)]
pub async fn request_with_auth(method: Method, uri: &str, token: &str) -> Response<Body> {
    let request = get_request_with_auth(method, uri, token);
    let app = create_test_router().await;

    app.oneshot(request.await).await.unwrap()
}

#[allow(dead_code)]
pub async fn request_with_auth_raw(
    method: Method,
    uri: &str,
    token: &str,
    content_type: &str,
    payload: Vec<u8>,
) -> Response<Body> {
    let request = get_request_with_auth_raw(method, uri, token, content_type, payload);
    let app = create_test_router().await;

    app.oneshot(request.await).await.unwrap()
}

#[allow(dead_code)]
pub async fn login_and_get_token() -> String {
    let payload = AuthPayload {
        client_id: TEST_CLIENT_ID.to_string(),
        client_secret: TEST_CLIENT_SECRET.to_string(),
    };

    let response = request_with_body(Method::POST, "/auth/login", &payload).await;
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, axum::http::StatusCode::OK);

    let response_body: RestApiResponse<AuthBody> = deserialize_json_body(body).await.unwrap();
    response_body.0.data.unwrap().access_token
}

/// internal helper functions to create requests
async fn get_request(method: Method, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri.to_string())
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .body(axum::body::Body::empty())
        .unwrap()
}

/// internal helper function to create a request with a body
async fn get_request_with_body(method: Method, uri: &str, payload: &str) -> Request<Body> {
    let request: Request<Body> = Request::builder()
        .method(method)
        .uri(uri.to_string())
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .body(axum::body::Body::from(payload.to_string()))
        .unwrap();

    request
}

async fn get_request_with_auth_body(
    method: Method,
    uri: &str,
    token: &str,
    payload: &str,
) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri.to_string())
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .body(axum::body::Body::from(payload.to_string()))
        .unwrap()
}

async fn get_request_with_auth(method: Method, uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri.to_string())
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .body(axum::body::Body::empty())
        .unwrap()
}

async fn get_request_with_auth_raw(
    method: Method,
    uri: &str,
    token: &str,
    content_type: &str,
    payload: Vec<u8>,
) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri.to_string())
        .header(CONTENT_TYPE, content_type)
        .header(ACCEPT, "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .body(axum::body::Body::from(payload))
        .unwrap()
}

async fn ensure_test_fixtures(pool: &Pool) -> TestResult<()> {
    let client = pool.get().await?;
    let password_hash = hash_utils::hash_password(TEST_CLIENT_SECRET)
        .map_err(|_| "Failed to hash test password")?;
    let auth_user_id = Uuid::parse_str(TEST_AUTH_USER_ID)?;
    let town_id = Uuid::parse_str(TEST_TOWN_ID)?;
    let category_id = Uuid::parse_str(TEST_CATEGORY_ID)?;

    let user_stmt = client
        .prepare_cached(
            r#"
            INSERT INTO users (id, username, email)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE
            SET
                username = EXCLUDED.username,
                email = EXCLUDED.email,
                modified_at = NOW()
            "#,
        )
        .await?;
    client
        .execute(
            &user_stmt,
            &[
                &auth_user_id,
                &TEST_CLIENT_ID,
                &format!("{}@example.com", TEST_CLIENT_ID),
            ],
        )
        .await?;

    let auth_stmt = client
        .prepare_cached(
            r#"
            INSERT INTO user_auth (user_id, password_hash)
            VALUES ($1, $2)
            ON CONFLICT (user_id) DO UPDATE
            SET
                password_hash = EXCLUDED.password_hash,
                modified_at = NOW()
            "#,
        )
        .await?;
    client
        .execute(&auth_stmt, &[&auth_user_id, &password_hash])
        .await?;

    let town_stmt = client
        .prepare_cached(
            r#"
            INSERT INTO towns (town_id, name)
            VALUES ($1, $2)
            ON CONFLICT (town_id) DO UPDATE
            SET name = EXCLUDED.name
            "#,
        )
        .await?;
    client
        .execute(&town_stmt, &[&town_id, &"Test Town"])
        .await?;

    let category_stmt = client
        .prepare_cached(
            r#"
            INSERT INTO categories (category_id, name)
            VALUES ($1, $2)
            ON CONFLICT (category_id) DO UPDATE
            SET name = EXCLUDED.name
            "#,
        )
        .await?;
    client
        .execute(&category_stmt, &[&category_id, &"Test Category"])
        .await?;

    Ok(())
}
