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
use utoipa::{
    OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

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
    ),
    security(
        ("bearer_auth" = [])
    ),
    modifiers(&BookingApiDoc)
)]
pub struct BookingApiDoc;

impl utoipa::Modify for BookingApiDoc {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some("Input your `<your‑jwt>`"))
                    .build(),
            ),
        )
    }
}

pub fn protected_booking_routes() -> Router<AppState> {
    Router::new()
        .route("/{article_id}/bookings", get(get_bookings))
        .route("/{article_id}/bookings", post(create_booking))
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
