use super::handlers::*;
use crate::{
    common::app_state::AppState,
    domains::calendar::{
        domain::model::{CalendarBlockReason, CalendarEntrySource, CalendarEntryType},
        dto::calendar_dto::{CalendarEntryDto, CreateCalendarEntryDto, UpdateCalendarEntryDto},
    },
};
use axum::{
    Router,
    routing::{delete, get, post, put},
};
use utoipa::{
    OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        get_calendar_entries,
        get_calendar_entry,
        create_calendar_entry,
        update_calendar_entry,
        delete_calendar_entry,
    ),
    components(schemas(
        CalendarBlockReason,
        CalendarEntryDto,
        CalendarEntrySource,
        CalendarEntryType,
        CreateCalendarEntryDto,
        UpdateCalendarEntryDto,
    )),
    tags(
        (name = "Calendar", description = "Article calendar entry endpoints")
    ),
    security(
        ("bearer_auth" = [])
    ),
    modifiers(&CalendarApiDoc)
)]
pub struct CalendarApiDoc;

impl utoipa::Modify for CalendarApiDoc {
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

pub fn protected_calendar_routes() -> Router<AppState> {
    Router::new()
        .route("/{article_id}/calendar", get(get_calendar_entries))
        .route("/{article_id}/calendar", post(create_calendar_entry))
        .route("/{article_id}/calendar/{event_id}", get(get_calendar_entry))
        .route(
            "/{article_id}/calendar/{event_id}",
            put(update_calendar_entry),
        )
        .route(
            "/{article_id}/calendar/{event_id}",
            delete(delete_calendar_entry),
        )
}
