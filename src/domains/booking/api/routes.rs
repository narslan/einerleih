use super::handlers::*;
use crate::{
    common::app_state::AppState,
    domains::booking::{
        BookingStatus,
        dto::booking_dto::{BookingDto, CreateBookingDto, UpdateBookingDto},
    },
};
use axum::{
    Router,
    routing::{get, post, put},
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_bookings,
        get_booking,
        create_booking,
        update_booking,
        confirm_booking,
        reject_booking,
        cancel_booking,
        complete_booking,
    ),
    components(schemas(
        BookingDto,
        BookingStatus,
        CreateBookingDto,
        UpdateBookingDto,
    )),
    tags(
        (name = "Bookings", description = "Article booking and reservation endpoints")
    )
)]
pub struct BookingApiDoc;

pub fn user_booking_routes() -> Router<AppState> {
    Router::new()
        .route("/{article_id}/bookings", post(create_booking))
        .route("/mine/{article_id}/bookings", get(get_own_bookings))
        .route(
            "/mine/{article_id}/bookings/{booking_id}",
            get(get_own_booking),
        )
        .route(
            "/mine/{article_id}/bookings/{booking_id}",
            put(update_own_booking),
        )
        .route(
            "/mine/{article_id}/bookings/{booking_id}/confirm",
            post(confirm_own_booking),
        )
        .route(
            "/mine/{article_id}/bookings/{booking_id}/reject",
            post(reject_own_booking),
        )
        .route(
            "/mine/{article_id}/bookings/{booking_id}/cancel",
            post(cancel_own_booking),
        )
        .route(
            "/mine/{article_id}/bookings/{booking_id}/complete",
            post(complete_own_booking),
        )
}

pub fn admin_booking_routes() -> Router<AppState> {
    Router::new()
        .route("/{article_id}/bookings", get(get_bookings))
        .route("/{article_id}/bookings/{booking_id}", get(get_booking))
        .route("/{article_id}/bookings/{booking_id}", put(update_booking))
        .route(
            "/{article_id}/bookings/{booking_id}/confirm",
            post(confirm_booking),
        )
        .route(
            "/{article_id}/bookings/{booking_id}/reject",
            post(reject_booking),
        )
        .route(
            "/{article_id}/bookings/{booking_id}/cancel",
            post(cancel_booking),
        )
        .route(
            "/{article_id}/bookings/{booking_id}/complete",
            post(complete_booking),
        )
}

pub fn protected_booking_routes() -> Router<AppState> {
    user_booking_routes().merge(admin_booking_routes())
}
