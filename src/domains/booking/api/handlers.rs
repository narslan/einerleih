use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    common::{
        app_state::AppState, auth_context::AuthenticatedUser, dto::RestApiResponse, error::AppError,
    },
    domains::booking::dto::booking_dto::{
        BookingDto, BookingFilterDto, CreateBookingDto, UpdateBookingDto,
    },
};

#[utoipa::path(
    get,
    path = "/article/{article_id}/bookings",
    params(
        ("article_id" = Uuid, Path, description = "Article ID"),
        BookingFilterDto
    ),
    responses((status = 200, description = "List bookings for article", body = [BookingDto])),
    tag = "Bookings"
)]
pub async fn get_bookings(
    State(state): State<AppState>,
    Path(article_id): Path<Uuid>,
    Query(filter): Query<BookingFilterDto>,
) -> Result<impl IntoResponse, AppError> {
    let bookings = state
        .booking_service
        .get_bookings_for_article(article_id, filter)
        .await?;
    Ok(RestApiResponse::success(bookings))
}

pub async fn get_own_bookings(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(article_id): Path<Uuid>,
    Query(filter): Query<BookingFilterDto>,
) -> Result<impl IntoResponse, AppError> {
    ensure_article_owner(&state, article_id, auth.id).await?;

    let bookings = state
        .booking_service
        .get_bookings_for_article(article_id, filter)
        .await?;
    Ok(RestApiResponse::success(bookings))
}

#[utoipa::path(
    get,
    path = "/article/{article_id}/bookings/{booking_id}",
    params(
        ("article_id" = Uuid, Path, description = "Article ID"),
        ("booking_id" = Uuid, Path, description = "Booking ID")
    ),
    responses((status = 200, description = "Get booking", body = BookingDto)),
    tag = "Bookings"
)]
pub async fn get_booking(
    State(state): State<AppState>,
    Path((article_id, booking_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let booking = state
        .booking_service
        .get_booking(article_id, booking_id)
        .await?;
    Ok(RestApiResponse::success(booking))
}

pub async fn get_own_booking(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path((article_id, booking_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    ensure_article_owner(&state, article_id, auth.id).await?;

    let booking = state
        .booking_service
        .get_booking(article_id, booking_id)
        .await?;
    Ok(RestApiResponse::success(booking))
}

#[utoipa::path(
    post,
    path = "/article/{article_id}/bookings",
    params(("article_id" = Uuid, Path, description = "Article ID")),
    request_body = CreateBookingDto,
    responses((status = 200, description = "Create booking request", body = BookingDto)),
    tag = "Bookings"
)]
pub async fn create_booking(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(article_id): Path<Uuid>,
    Json(payload): Json<CreateBookingDto>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(AppError::from)?;

    let mut payload = payload;
    payload.created_by = auth.id;
    payload.modified_by = auth.id;
    payload.requested_by = Some(auth.id);

    let article = state.article_service.get_article_by_id(article_id).await?;
    if article.created_by == Some(auth.id) {
        return Err(AppError::ValidationError(
            "You cannot request a booking for your own article".into(),
        ));
    }

    let booking = state
        .booking_service
        .create_booking(article_id, payload)
        .await?;
    Ok(RestApiResponse::success(booking))
}

#[utoipa::path(
    put,
    path = "/article/{article_id}/bookings/{booking_id}",
    params(
        ("article_id" = Uuid, Path, description = "Article ID"),
        ("booking_id" = Uuid, Path, description = "Booking ID")
    ),
    request_body = UpdateBookingDto,
    responses((status = 200, description = "Update booking", body = BookingDto)),
    tag = "Bookings"
)]
pub async fn update_booking(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path((article_id, booking_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateBookingDto>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(AppError::from)?;

    let mut payload = payload;
    payload.modified_by = auth.id;

    let booking = state
        .booking_service
        .update_booking(article_id, booking_id, payload)
        .await?;
    Ok(RestApiResponse::success(booking))
}

pub async fn update_own_booking(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path((article_id, booking_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateBookingDto>,
) -> Result<impl IntoResponse, AppError> {
    ensure_article_owner(&state, article_id, auth.id).await?;

    payload.validate().map_err(AppError::from)?;

    let mut payload = payload;
    payload.modified_by = auth.id;

    let booking = state
        .booking_service
        .update_booking(article_id, booking_id, payload)
        .await?;
    Ok(RestApiResponse::success(booking))
}

#[utoipa::path(
    post,
    path = "/article/{article_id}/bookings/{booking_id}/confirm",
    params(
        ("article_id" = Uuid, Path, description = "Article ID"),
        ("booking_id" = Uuid, Path, description = "Booking ID")
    ),
    responses((status = 200, description = "Confirm booking", body = BookingDto)),
    tag = "Bookings"
)]
pub async fn confirm_booking(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path((article_id, booking_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let booking = state
        .booking_service
        .confirm_booking(article_id, booking_id, auth.id)
        .await?;
    Ok(RestApiResponse::success(booking))
}

pub async fn confirm_own_booking(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path((article_id, booking_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    ensure_article_owner(&state, article_id, auth.id).await?;

    let booking = state
        .booking_service
        .confirm_booking(article_id, booking_id, auth.id)
        .await?;
    Ok(RestApiResponse::success(booking))
}

#[utoipa::path(
    post,
    path = "/article/{article_id}/bookings/{booking_id}/reject",
    params(
        ("article_id" = Uuid, Path, description = "Article ID"),
        ("booking_id" = Uuid, Path, description = "Booking ID")
    ),
    responses((status = 200, description = "Reject booking", body = BookingDto)),
    tag = "Bookings"
)]
pub async fn reject_booking(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path((article_id, booking_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let booking = state
        .booking_service
        .reject_booking(article_id, booking_id, auth.id)
        .await?;
    Ok(RestApiResponse::success(booking))
}

pub async fn reject_own_booking(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path((article_id, booking_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    ensure_article_owner(&state, article_id, auth.id).await?;

    let booking = state
        .booking_service
        .reject_booking(article_id, booking_id, auth.id)
        .await?;
    Ok(RestApiResponse::success(booking))
}

#[utoipa::path(
    post,
    path = "/article/{article_id}/bookings/{booking_id}/cancel",
    params(
        ("article_id" = Uuid, Path, description = "Article ID"),
        ("booking_id" = Uuid, Path, description = "Booking ID")
    ),
    responses((status = 200, description = "Cancel booking", body = BookingDto)),
    tag = "Bookings"
)]
pub async fn cancel_booking(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path((article_id, booking_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let booking = state
        .booking_service
        .cancel_booking(article_id, booking_id, auth.id)
        .await?;
    Ok(RestApiResponse::success(booking))
}

pub async fn cancel_own_booking(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path((article_id, booking_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    ensure_article_owner(&state, article_id, auth.id).await?;

    let booking = state
        .booking_service
        .cancel_booking(article_id, booking_id, auth.id)
        .await?;
    Ok(RestApiResponse::success(booking))
}

#[utoipa::path(
    post,
    path = "/article/{article_id}/bookings/{booking_id}/complete",
    params(
        ("article_id" = Uuid, Path, description = "Article ID"),
        ("booking_id" = Uuid, Path, description = "Booking ID")
    ),
    responses((status = 200, description = "Complete booking", body = BookingDto)),
    tag = "Bookings"
)]
pub async fn complete_booking(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path((article_id, booking_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let booking = state
        .booking_service
        .complete_booking(article_id, booking_id, auth.id)
        .await?;
    Ok(RestApiResponse::success(booking))
}

pub async fn complete_own_booking(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path((article_id, booking_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    ensure_article_owner(&state, article_id, auth.id).await?;

    let booking = state
        .booking_service
        .complete_booking(article_id, booking_id, auth.id)
        .await?;
    Ok(RestApiResponse::success(booking))
}

async fn ensure_article_owner(
    state: &AppState,
    article_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let article = state.article_service.get_article_by_id(article_id).await?;
    if article.created_by == Some(user_id) {
        return Ok(());
    }

    Err(AppError::Forbidden)
}
