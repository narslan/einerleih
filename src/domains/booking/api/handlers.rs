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
    payload
        .validate()
        .map_err(|err| AppError::ValidationError(format!("Invalid input: {}", err)))?;

    let mut payload = payload;
    payload.created_by = auth.id;
    payload.modified_by = auth.id;
    if payload.requested_by.is_none() {
        payload.requested_by = Some(auth.id);
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
    payload
        .validate()
        .map_err(|err| AppError::ValidationError(format!("Invalid input: {}", err)))?;

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
