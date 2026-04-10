use axum::http::{Method, StatusCode};
use chrono::{TimeZone, Utc};

use einerleih::{
    common::{dto::RestApiResponse, error::AppError},
    domains::{
        article::{
            ArticleStatus,
            dto::article_dto::{ArticleDto, CreateArticleDto},
        },
        calendar::{
            CalendarBlockReason, CalendarEntrySource, CalendarEntryType,
            dto::calendar_dto::{CalendarEntryDto, CreateCalendarEntryDto},
        },
    },
};

mod test_helpers;

use test_helpers::{
    TEST_CATEGORY_ID, TEST_TOWN_ID, deserialize_json_body, login_and_get_token, request_with_auth,
    request_with_auth_body,
};

async fn create_article() -> Result<ArticleDto, AppError> {
    let uuid_text = uuid::Uuid::new_v4().simple().to_string();
    let short_id = &uuid_text[..8];
    let payload = CreateArticleDto {
        name: format!("cal-{short_id}"),
        category: uuid::Uuid::parse_str(TEST_CATEGORY_ID).unwrap(),
        description: "Artikel fuer Kalender-Tests".to_string(),
        town: uuid::Uuid::parse_str(TEST_TOWN_ID).unwrap(),
        status: ArticleStatus::Aktiv,
        created_by: uuid::Uuid::nil(),
        modified_by: uuid::Uuid::nil(),
    };

    let token = login_and_get_token().await;
    let response = request_with_auth_body(Method::POST, "/article", &token, &payload).await;
    let (parts, body) = response.into_parts();
    let response_body: RestApiResponse<ArticleDto> = deserialize_json_body(body).await.unwrap();
    if parts.status != StatusCode::OK {
        panic!(
            "expected 200 from POST /article, got {} with body {:?}",
            parts.status, response_body.0
        );
    }

    Ok(response_body.0.data.unwrap())
}

fn create_availability_entry_payload() -> CreateCalendarEntryDto {
    CreateCalendarEntryDto {
        entry_type: CalendarEntryType::Availability,
        block_reason: None,
        summary: "Buchbar am Vormittag".to_string(),
        location: Some("Abholung im Laden".to_string()),
        description: Some("Regulaeres Verfuegbarkeitsfenster".to_string()),
        start_time: Utc.with_ymd_and_hms(2026, 5, 1, 8, 0, 0).unwrap(),
        end_time: Utc.with_ymd_and_hms(2026, 5, 1, 12, 0, 0).unwrap(),
        rrule: None,
        source: CalendarEntrySource::Manual,
        created_by: uuid::Uuid::nil(),
        modified_by: uuid::Uuid::nil(),
    }
}

#[tokio::test]
async fn test_create_and_list_article_calendar_entry() {
    let article = create_article()
        .await
        .expect("Failed to create article for calendar test");
    let token = login_and_get_token().await;
    let payload = create_availability_entry_payload();

    let response = request_with_auth_body(
        Method::POST,
        &format!("/article/{}/calendar", article.article_id),
        &token,
        &payload,
    )
    .await;
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<CalendarEntryDto> =
        deserialize_json_body(body).await.unwrap();
    let created_entry = response_body.0.data.unwrap();
    assert_eq!(created_entry.article_id, article.article_id);
    assert_eq!(created_entry.entry_type, CalendarEntryType::Availability);
    assert_eq!(created_entry.block_reason, None);
    assert_eq!(created_entry.summary, payload.summary);

    let response = request_with_auth(
        Method::GET,
        &format!("/article/{}/calendar", article.article_id),
        &token,
    )
    .await;
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<Vec<CalendarEntryDto>> =
        deserialize_json_body(body).await.unwrap();
    let entries = response_body.0.data.unwrap();
    assert!(
        entries
            .iter()
            .any(|entry| entry.event_id == created_entry.event_id)
    );
}

#[tokio::test]
async fn test_rejects_invalid_calendar_entry_semantics() {
    let article = create_article()
        .await
        .expect("Failed to create article for calendar test");
    let token = login_and_get_token().await;
    let mut payload = create_availability_entry_payload();
    payload.block_reason = Some(CalendarBlockReason::Repair);

    let response = request_with_auth_body(
        Method::POST,
        &format!("/article/{}/calendar", article.article_id),
        &token,
        &payload,
    )
    .await;
    let (parts, _) = response.into_parts();
    assert_eq!(parts.status, StatusCode::BAD_REQUEST);
}
