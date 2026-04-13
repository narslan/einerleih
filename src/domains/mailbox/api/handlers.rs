use axum::{
    Extension,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    common::{
        app_state::AppState, auth_context::AuthenticatedUser, dto::RestApiResponse, error::AppError,
    },
    domains::mailbox::dto::mailbox_dto::{MailboxEntryDto, MailboxEntryFilterDto},
};

#[utoipa::path(
    get,
    path = "/mailbox",
    params(MailboxEntryFilterDto),
    responses((status = 200, description = "List mailbox entries for current user", body = [MailboxEntryDto])),
    tag = "Mailbox"
)]
pub async fn get_mailbox_entries(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Query(filter): Query<MailboxEntryFilterDto>,
) -> Result<impl IntoResponse, AppError> {
    let entries = state.mailbox_service.get_entries(auth.id, filter).await?;
    Ok(RestApiResponse::success(entries))
}

#[utoipa::path(
    get,
    path = "/mailbox/{mailbox_entry_id}",
    params(("mailbox_entry_id" = Uuid, Path, description = "Mailbox entry ID")),
    responses((status = 200, description = "Get mailbox entry for current user", body = MailboxEntryDto)),
    tag = "Mailbox"
)]
pub async fn get_mailbox_entry(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(mailbox_entry_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let entry = state
        .mailbox_service
        .get_entry(auth.id, mailbox_entry_id)
        .await?;
    Ok(RestApiResponse::success(entry))
}

#[utoipa::path(
    post,
    path = "/mailbox/{mailbox_entry_id}/read",
    params(("mailbox_entry_id" = Uuid, Path, description = "Mailbox entry ID")),
    responses((status = 200, description = "Mark mailbox entry as read", body = MailboxEntryDto)),
    tag = "Mailbox"
)]
pub async fn mark_mailbox_entry_as_read(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(mailbox_entry_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let entry = state
        .mailbox_service
        .mark_as_read(auth.id, mailbox_entry_id)
        .await?;
    Ok(RestApiResponse::success(entry))
}
