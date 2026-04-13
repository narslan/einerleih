use super::handlers::*;
use crate::{
    common::app_state::AppState,
    domains::calendar::{
        domain::model::CalendarEntrySource,
        dto::calendar_dto::{CalendarEntryDto, CreateCalendarEntryDto, UpdateCalendarEntryDto},
    },
};
use axum::{
    Router,
    routing::{delete, get, post, put},
};
use utoipa::OpenApi;

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
        CalendarEntryDto,
        CalendarEntrySource,
        CreateCalendarEntryDto,
        UpdateCalendarEntryDto,
    )),
    tags(
        (name = "Calendar", description = "Article calendar entry endpoints")
    )
)]
pub struct CalendarApiDoc;

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

pub fn user_calendar_routes() -> Router<AppState> {
    Router::new()
        .route("/mine/{article_id}/calendar", get(get_own_calendar_entries))
        .route(
            "/mine/{article_id}/calendar",
            post(create_own_calendar_entry),
        )
        .route(
            "/mine/{article_id}/calendar/{event_id}",
            get(get_own_calendar_entry),
        )
        .route(
            "/mine/{article_id}/calendar/{event_id}",
            put(update_own_calendar_entry),
        )
        .route(
            "/mine/{article_id}/calendar/{event_id}",
            delete(delete_own_calendar_entry),
        )
}
