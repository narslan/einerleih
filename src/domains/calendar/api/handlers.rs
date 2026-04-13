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
    domains::calendar::dto::calendar_dto::{
        CalendarEntryDto, CalendarEntryFilterDto, CreateCalendarEntryDto, UpdateCalendarEntryDto,
    },
};

#[utoipa::path(
    get,
    path = "/article/{article_id}/calendar",
    params(
        ("article_id" = Uuid, Path, description = "Article ID"),
        CalendarEntryFilterDto
    ),
    responses((status = 200, description = "List calendar entries for article", body = [CalendarEntryDto])),
    tag = "Calendar"
)]
pub async fn get_calendar_entries(
    State(state): State<AppState>,
    Path(article_id): Path<Uuid>,
    Query(filter): Query<CalendarEntryFilterDto>,
) -> Result<impl IntoResponse, AppError> {
    let entries = state
        .calendar_service
        .get_entries_for_article(article_id, filter)
        .await?;
    Ok(RestApiResponse::success(entries))
}

pub async fn get_own_calendar_entries(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(article_id): Path<Uuid>,
    Query(filter): Query<CalendarEntryFilterDto>,
) -> Result<impl IntoResponse, AppError> {
    ensure_article_owner(&state, article_id, auth.id).await?;

    let entries = state
        .calendar_service
        .get_entries_for_article(article_id, filter)
        .await?;
    Ok(RestApiResponse::success(entries))
}

#[utoipa::path(
    get,
    path = "/article/{article_id}/calendar/{event_id}",
    params(
        ("article_id" = Uuid, Path, description = "Article ID"),
        ("event_id" = Uuid, Path, description = "Calendar entry ID")
    ),
    responses((status = 200, description = "Get calendar entry", body = CalendarEntryDto)),
    tag = "Calendar"
)]
pub async fn get_calendar_entry(
    State(state): State<AppState>,
    Path((article_id, event_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let entry = state
        .calendar_service
        .get_entry(article_id, event_id)
        .await?;
    Ok(RestApiResponse::success(entry))
}

pub async fn get_own_calendar_entry(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path((article_id, event_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    ensure_article_owner(&state, article_id, auth.id).await?;

    let entry = state
        .calendar_service
        .get_entry(article_id, event_id)
        .await?;
    Ok(RestApiResponse::success(entry))
}

#[utoipa::path(
    post,
    path = "/article/{article_id}/calendar",
    params(("article_id" = Uuid, Path, description = "Article ID")),
    request_body = CreateCalendarEntryDto,
    responses((status = 200, description = "Create calendar entry", body = CalendarEntryDto)),
    tag = "Calendar"
)]
pub async fn create_calendar_entry(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(article_id): Path<Uuid>,
    Json(payload): Json<CreateCalendarEntryDto>,
) -> Result<impl IntoResponse, AppError> {
    payload
        .validate()
        .map_err(|err| AppError::ValidationError(format!("Invalid input: {}", err)))?;

    let mut payload = payload;
    payload.created_by = auth.id;
    payload.modified_by = auth.id;

    let entry = state
        .calendar_service
        .create_entry(article_id, payload)
        .await?;
    Ok(RestApiResponse::success(entry))
}

pub async fn create_own_calendar_entry(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(article_id): Path<Uuid>,
    Json(payload): Json<CreateCalendarEntryDto>,
) -> Result<impl IntoResponse, AppError> {
    ensure_article_owner(&state, article_id, auth.id).await?;

    payload
        .validate()
        .map_err(|err| AppError::ValidationError(format!("Invalid input: {}", err)))?;

    let mut payload = payload;
    payload.created_by = auth.id;
    payload.modified_by = auth.id;

    let entry = state
        .calendar_service
        .create_entry(article_id, payload)
        .await?;
    Ok(RestApiResponse::success(entry))
}

#[utoipa::path(
    put,
    path = "/article/{article_id}/calendar/{event_id}",
    params(
        ("article_id" = Uuid, Path, description = "Article ID"),
        ("event_id" = Uuid, Path, description = "Calendar entry ID")
    ),
    request_body = UpdateCalendarEntryDto,
    responses((status = 200, description = "Update calendar entry", body = CalendarEntryDto)),
    tag = "Calendar"
)]
pub async fn update_calendar_entry(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path((article_id, event_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateCalendarEntryDto>,
) -> Result<impl IntoResponse, AppError> {
    payload
        .validate()
        .map_err(|err| AppError::ValidationError(format!("Invalid input: {}", err)))?;

    let mut payload = payload;
    payload.modified_by = auth.id;

    let entry = state
        .calendar_service
        .update_entry(article_id, event_id, payload)
        .await?;
    Ok(RestApiResponse::success(entry))
}

pub async fn update_own_calendar_entry(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path((article_id, event_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateCalendarEntryDto>,
) -> Result<impl IntoResponse, AppError> {
    ensure_article_owner(&state, article_id, auth.id).await?;

    payload
        .validate()
        .map_err(|err| AppError::ValidationError(format!("Invalid input: {}", err)))?;

    let mut payload = payload;
    payload.modified_by = auth.id;

    let entry = state
        .calendar_service
        .update_entry(article_id, event_id, payload)
        .await?;
    Ok(RestApiResponse::success(entry))
}

#[utoipa::path(
    delete,
    path = "/article/{article_id}/calendar/{event_id}",
    params(
        ("article_id" = Uuid, Path, description = "Article ID"),
        ("event_id" = Uuid, Path, description = "Calendar entry ID")
    ),
    responses((status = 200, description = "Calendar entry deleted")),
    tag = "Calendar"
)]
pub async fn delete_calendar_entry(
    State(state): State<AppState>,
    Path((article_id, event_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let message = state
        .calendar_service
        .delete_entry(article_id, event_id)
        .await?;
    Ok(RestApiResponse::success_with_message(message, ()))
}

pub async fn delete_own_calendar_entry(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path((article_id, event_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    ensure_article_owner(&state, article_id, auth.id).await?;

    let message = state
        .calendar_service
        .delete_entry(article_id, event_id)
        .await?;
    Ok(RestApiResponse::success_with_message(message, ()))
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
