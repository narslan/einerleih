use axum::http::{Method, StatusCode};
use chrono::NaiveDate;

use einerleih::{
    common::{dto::RestApiResponse, error::AppError},
    domains::{
        article::{
            ArticleStatus,
            dto::article_dto::{
                ArticleDto, CreateArticleDto, CreateArticleWithPicturesResponseDto,
            },
        },
        calendar::{
            CalendarEntrySource,
            dto::calendar_dto::{CalendarEntryDto, CreateCalendarEntryDto},
        },
    },
};

mod test_helpers;

use test_helpers::{
    TEST_CATEGORY_ID, TEST_TOWN_ID, deserialize_json_body, login_and_get_session_cookie,
    request_with_session, request_with_session_body, request_with_session_raw,
    signup_and_get_session_cookie,
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
        tags: Vec::new(),
        created_by: uuid::Uuid::nil(),
        modified_by: uuid::Uuid::nil(),
    };

    let boundary = format!("boundary-{}", uuid::Uuid::new_v4().simple());
    let mut body = Vec::new();
    append_multipart_text_part(&mut body, &boundary, "name", &payload.name);
    append_multipart_text_part(
        &mut body,
        &boundary,
        "category",
        &payload.category.to_string(),
    );
    append_multipart_text_part(&mut body, &boundary, "description", &payload.description);
    append_multipart_text_part(&mut body, &boundary, "town", &payload.town.to_string());
    append_multipart_text_part(&mut body, &boundary, "status", payload.status.as_str());
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

    let session_cookie = login_and_get_session_cookie().await;
    let response = request_with_session_raw(
        Method::POST,
        "/article/mine/upload",
        &session_cookie,
        &format!("multipart/form-data; boundary={boundary}"),
        body,
    )
    .await;
    let (parts, body) = response.into_parts();
    let response_body: RestApiResponse<CreateArticleWithPicturesResponseDto> =
        deserialize_json_body(body).await.unwrap();
    if parts.status != StatusCode::OK {
        panic!(
            "expected 200 from POST /article/mine/upload, got {} with body {:?}",
            parts.status, response_body.0
        );
    }

    Ok(response_body.0.data.unwrap().article)
}

fn append_multipart_text_part(body: &mut Vec<u8>, boundary: &str, name: &str, value: &str) {
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
    );
    body.extend_from_slice(value.as_bytes());
    body.extend_from_slice(b"\r\n");
}

fn create_calendar_entry_payload() -> CreateCalendarEntryDto {
    CreateCalendarEntryDto {
        start_date: NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2026, 5, 3).unwrap(),
        location: Some("Abholung im Laden".to_string()),
        description: Some("Regulaerer Kalendereintrag".to_string()),
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
    let session_cookie = login_and_get_session_cookie().await;
    let payload = create_calendar_entry_payload();

    let response = request_with_session_body(
        Method::POST,
        &format!("/article/{}/calendar", article.article_id),
        &session_cookie,
        &payload,
    )
    .await;
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<CalendarEntryDto> =
        deserialize_json_body(body).await.unwrap();
    let created_entry = response_body.0.data.unwrap();
    assert_eq!(created_entry.article_id, article.article_id);
    assert_eq!(created_entry.start_date, payload.start_date);
    assert_eq!(created_entry.end_date, payload.end_date);
    assert_eq!(created_entry.location, payload.location);
    assert_eq!(created_entry.description, payload.description);
    assert_eq!(created_entry.source, CalendarEntrySource::Manual);

    let response = request_with_session(
        Method::GET,
        &format!("/article/{}/calendar", article.article_id),
        &session_cookie,
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
async fn test_article_owner_can_manage_calendar_via_mine_routes() {
    let article = create_article()
        .await
        .expect("Failed to create article for owner calendar test");
    let owner_session_cookie = login_and_get_session_cookie().await;
    let other_session_cookie = signup_and_get_session_cookie().await;
    let payload = create_calendar_entry_payload();

    let response = request_with_session_body(
        Method::POST,
        &format!("/article/mine/{}/calendar", article.article_id),
        &owner_session_cookie,
        &payload,
    )
    .await;
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, StatusCode::OK);
    let response_body: RestApiResponse<CalendarEntryDto> =
        deserialize_json_body(body).await.unwrap();
    let created_entry = response_body.0.data.unwrap();

    let response = request_with_session(
        Method::GET,
        &format!("/article/mine/{}/calendar", article.article_id),
        &owner_session_cookie,
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

    let response = request_with_session(
        Method::DELETE,
        &format!(
            "/article/mine/{}/calendar/{}",
            article.article_id, created_entry.event_id
        ),
        &other_session_cookie,
    )
    .await;
    assert_eq!(response.into_parts().0.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_rejects_invalid_calendar_date_range() {
    let article = create_article()
        .await
        .expect("Failed to create article for calendar test");
    let session_cookie = login_and_get_session_cookie().await;
    let mut payload = create_calendar_entry_payload();
    payload.start_date = NaiveDate::from_ymd_opt(2026, 5, 3).unwrap();
    payload.end_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();

    let response = request_with_session_body(
        Method::POST,
        &format!("/article/{}/calendar", article.article_id),
        &session_cookie,
        &payload,
    )
    .await;
    let (parts, _) = response.into_parts();
    assert_eq!(parts.status, StatusCode::BAD_REQUEST);
}
