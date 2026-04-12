use axum::{
    Router,
    body::{Body, Bytes},
    error_handling::HandleErrorLayer,
    extract::Request,
    http::{Method, StatusCode, header::CONTENT_TYPE},
    middleware::{self, Next},
    response::Response,
};
use axum_login::AuthManagerLayerBuilder;
use http_body_util::BodyExt;
use tower_sessions::{MemoryStore, SessionManagerLayer};

use std::{sync::LazyLock, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use crate::{
    common::{app_state::AppState, error::handle_error, session_auth},
    domains::{
        article::{ArticleApiDoc, protected_article_routes, public_article_routes},
        auth::{UserAuthApiDoc, user_auth_routes},
        booking::{BookingApiDoc, admin_booking_routes, user_booking_routes},
        calendar::{CalendarApiDoc, protected_calendar_routes},
        file::{FileApiDoc, protected_file_routes, public_file_routes},
        user::{UserApiDoc, user_routes},
    },
};
use utoipa::OpenApi;

use once_cell::sync::Lazy;
use regex::Regex;
use utoipa_swagger_ui::SwaggerUi;

/// List of regex patterns representing disallowed content to block in requests.
/// These patterns are applied to both request bodies and URL query strings.
/// Used to detect and reject potentially dangerous input (e.g., script tags).
/// This is just sample. In real app this can be loaded from repository
pub static FORBIDDEN_PATTERNS: Lazy<Vec<Regex>> =
    Lazy::new(|| vec![Regex::new(r"(?i)<\s*script\b[^>]*>").unwrap()]);

static SESSION_STORE: LazyLock<MemoryStore> = LazyLock::new(MemoryStore::default);

fn create_swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/docs")
        .url(
            "/api-docs/user_auth/openapi.json",
            UserAuthApiDoc::openapi(),
        )
        .url("/api-docs/user/openapi.json", UserApiDoc::openapi())
        .url("/api-docs/article/openapi.json", ArticleApiDoc::openapi())
        .url("/api-docs/calendar/openapi.json", CalendarApiDoc::openapi())
        .url("/api-docs/booking/openapi.json", BookingApiDoc::openapi())
        .url("/api-docs/file/openapi.json", FileApiDoc::openapi())
}

pub fn create_router(state: AppState) -> Router {
    let session_layer = SessionManagerLayer::new(SESSION_STORE.clone())
        .with_name(state.config.session_cookie_name.clone())
        .with_secure(state.config.session_cookie_secure);
    let auth_backend = session_auth::AuthBackend::new(state.pool.clone());
    let auth_layer = AuthManagerLayerBuilder::new(auth_backend, session_layer).build();

    // Build a CORS layer that applies to everyone
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any)
        .allow_headers([CONTENT_TYPE]);

    // Create a common middleware stack for error handling, timeouts, and CORS.
    let middleware_stack = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_error))
        .timeout(Duration::from_secs(1800))
        .layer(cors);

    // /auth routes (login, register, logout) — no logging here
    let auth_router = Router::new()
        .nest("/auth", user_auth_routes())
        .layer(middleware::from_fn(make_request_response_inspecter(false)));

    // Routes for regular authenticated users.
    let session_routes = Router::new()
        .nest("/article", user_booking_routes())
        .route_layer(middleware::from_fn(session_auth::require_session))
        .layer(middleware::from_fn(make_request_response_inspecter(true)));

    // Admin routes for managing articles, calendars, bookings, files and users.
    let admin_routes = Router::new()
        .nest("/user", user_routes())
        .nest(
            "/article",
            protected_article_routes()
                .merge(protected_calendar_routes())
                .merge(admin_booking_routes()),
        )
        .nest("/file", protected_file_routes())
        .route_layer(middleware::from_fn(session_auth::require_admin))
        .layer(middleware::from_fn(make_request_response_inspecter(true)));

    let public_article_routes = Router::new()
        .nest("/article", public_article_routes())
        .layer(middleware::from_fn(make_request_response_inspecter(false)));

    let public_file_routes = Router::new()
        .nest("/file", public_file_routes())
        .layer(middleware::from_fn(make_request_response_inspecter(false)));

    // setup assets routes
    let static_files = ServeDir::new("static/dist")
        .append_index_html_on_directories(true)
        .not_found_service(ServeFile::new("static/dist/index.html"));

    // Router::new()
    //     .route_service("/", ServeFile::new("static/dist/index.html"))
    //     .fallback_service(static_files)
    //     .layer(middleware_stack)
    //     .with_state(state)

    Router::new()
        .route("/health", axum::routing::get(health_check))
        .route_service("/", ServeFile::new("static/dist/index.html"))
        .route_service("/admin", ServeFile::new("static/dist/index.html"))
        .route_service("/admin/{*path}", ServeFile::new("static/dist/index.html"))
        .route_service("/katalog", ServeFile::new("static/dist/index.html"))
        .route_service("/katalog/{*path}", ServeFile::new("static/dist/index.html"))
        .merge(auth_router)
        .merge(public_article_routes)
        .merge(public_file_routes)
        .merge(session_routes)
        .merge(admin_routes)
        .merge(create_swagger_ui())
        .fallback_service(static_files)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &axum::http::Request<_>| {
                    tracing::info_span!(
                        "request",
                        method = %req.method(),
                        uri = %req.uri(),
                    )
                })
                .on_response(
                    |response: &axum::http::Response<_>,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        tracing::info!(
                            "request completed: status = {status}, latency = {latency:?}",
                            status = response.status(),
                            latency = latency
                        );
                    },
                ),
        )
        .layer(middleware_stack)
        .layer(auth_layer)
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK\n"
}

/// Middleware that inspects request bodies and URL query strings, as well as response bodies, logging them for debugging, and rejects forbidden content.
/// Intercepts HTTP requests and responses: buffers bodies and query strings, then logs their content.
/// Returns a 403 Forbidden error if any forbidden patterns are detected in the request body or query string.
/// Note: multipart/form-data requests bypass this middleware and must be validated within their handlers.
// Type alias for the boxed future returned by the request/response inspector middleware

type InspectorFuture = std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<Response, (StatusCode, String)>> + Send>,
>;

/// This function inspects forbidden request and collects the body data into bytes and prints it to the log.
async fn request_inspect_print<B>(
    direction: &str,
    log_enabled: bool,
    body: B,
) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {direction} body: {err}"),
            ));
        }
    };

    if let Ok(body_str) = std::str::from_utf8(&bytes) {
        if log_enabled {
            tracing::info!("{} body = {:?}", direction, body_str);
        }

        // inspect forbidden request body
        if FORBIDDEN_PATTERNS.iter().any(|re| re.is_match(body_str)) {
            return Err((StatusCode::FORBIDDEN, "Forbidden Request".to_string()));
        }
    }

    Ok(bytes)
}

async fn request_response_inspecter(
    req: Request<Body>,
    next: Next,
    log_enabled: bool,
) -> Result<Response, (StatusCode, String)> {
    // inspect forbidden query string
    if let Some(query) = req.uri().query() {
        if FORBIDDEN_PATTERNS.iter().any(|re| re.is_match(query)) {
            return Err((StatusCode::FORBIDDEN, "Forbidden Request".to_string()));
        }
    }

    let (parts, body) = req.into_parts();
    let bytes = request_inspect_print("request", log_enabled, body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let mut res = next.run(req).await;
    if log_enabled && tracing::enabled!(tracing::Level::DEBUG) {
        let (parts, body) = res.into_parts();
        let bytes = response_print("response", body).await?;
        res = Response::from_parts(parts, Body::from(bytes));
    }

    Ok(res)
}

async fn response_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {direction} body: {err}"),
            ));
        }
    };

    if let Ok(body_str) = std::str::from_utf8(&bytes) {
        tracing::debug!("{} body = {:?}", direction, body_str);
    }

    Ok(bytes)
}
fn make_request_response_inspecter(
    log_enabled: bool,
) -> impl Fn(Request<Body>, Next) -> InspectorFuture + Clone + Send + Sync + 'static {
    move |req, next| {
        let fut = request_response_inspecter(req, next, log_enabled);
        Box::pin(fut)
    }
}
