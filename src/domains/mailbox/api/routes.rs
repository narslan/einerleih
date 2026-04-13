use super::handlers::*;
use crate::{
    common::app_state::AppState,
    domains::mailbox::{MailboxDirection, dto::mailbox_dto::MailboxEntryDto},
};
use axum::{
    Router,
    routing::{get, post},
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_mailbox_entries,
        get_mailbox_entry,
        mark_mailbox_entry_as_read
    ),
    components(schemas(MailboxEntryDto, MailboxDirection)),
    tags((name = "Mailbox", description = "Current user mailbox endpoints"))
)]
pub struct MailboxApiDoc;

pub fn user_mailbox_routes() -> Router<AppState> {
    Router::new()
        .route("/mailbox", get(get_mailbox_entries))
        .route("/mailbox/{mailbox_entry_id}", get(get_mailbox_entry))
        .route(
            "/mailbox/{mailbox_entry_id}/read",
            post(mark_mailbox_entry_as_read),
        )
}
