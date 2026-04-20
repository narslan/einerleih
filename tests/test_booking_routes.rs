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
        booking::{
            BookingStatus,
            dto::booking_dto::{BookingDto, CreateBookingDto},
        },
        mailbox::{MailboxDirection, dto::mailbox_dto::MailboxEntryDto},
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
        name: format!("book-{short_id}"),
        category: uuid::Uuid::parse_str(TEST_CATEGORY_ID).unwrap(),
        description: "Artikel fuer Buchungs-Tests".to_string(),
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

fn create_booking_payload(start_day: u32, end_day: u32) -> CreateBookingDto {
    CreateBookingDto {
        requested_by: None,
        requester_name: Some("API Test".to_string()),
        requester_email: Some("apitest@example.com".to_string()),
        note: Some("Bitte reservieren".to_string()),
        start_date: NaiveDate::from_ymd_opt(2026, 5, start_day).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2026, 5, end_day).unwrap(),
        created_by: uuid::Uuid::nil(),
        modified_by: uuid::Uuid::nil(),
    }
}

async fn create_booking(
    article_id: uuid::Uuid,
    session_cookie: &str,
    payload: &CreateBookingDto,
) -> BookingDto {
    let response = request_with_session_body(
        Method::POST,
        &format!("/article/{article_id}/bookings"),
        session_cookie,
        payload,
    )
    .await;
    let (parts, body) = response.into_parts();
    let response_body: RestApiResponse<BookingDto> = deserialize_json_body(body).await.unwrap();
    if parts.status != StatusCode::OK {
        panic!(
            "expected 200 from POST /article/{article_id}/bookings, got {} with body {:?}",
            parts.status, response_body.0
        );
    }

    response_body.0.data.unwrap()
}

#[tokio::test]
async fn test_regular_user_can_create_but_not_administer_booking() {
    let article = create_article()
        .await
        .expect("Failed to create article for booking test");
    let user_session_cookie = signup_and_get_session_cookie().await;
    let payload = create_booking_payload(13, 16);

    let booking = create_booking(article.article_id, &user_session_cookie, &payload).await;
    assert_eq!(booking.status, BookingStatus::Requested);

    let response = request_with_session(
        Method::POST,
        &format!(
            "/article/{}/bookings/{}/confirm",
            article.article_id, booking.booking_id
        ),
        &user_session_cookie,
    )
    .await;
    assert_eq!(response.into_parts().0.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_booking_request_creates_mailbox_entries_for_requester_and_provider() {
    let article = create_article()
        .await
        .expect("Failed to create article for mailbox test");
    let owner_session_cookie = login_and_get_session_cookie().await;
    let requester_session_cookie = signup_and_get_session_cookie().await;
    let provider_id = article
        .created_by
        .expect("created article should store provider");

    let mut payload = create_booking_payload(13, 16);
    payload.requested_by = Some(provider_id);
    let booking = create_booking(article.article_id, &requester_session_cookie, &payload).await;
    let requester_id = booking
        .requested_by
        .expect("created booking should store requester");
    assert_ne!(requester_id, provider_id);

    let response = request_with_session(
        Method::GET,
        "/mailbox?direction=sent",
        &requester_session_cookie,
    )
    .await;
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, StatusCode::OK);
    let response_body: RestApiResponse<Vec<MailboxEntryDto>> =
        deserialize_json_body(body).await.unwrap();
    let sent_entries = response_body.0.data.unwrap();
    let sent_entry = sent_entries
        .iter()
        .find(|entry| entry.booking_id == Some(booking.booking_id))
        .expect("requester mailbox should contain sent booking request");
    assert_eq!(sent_entry.owner_id, requester_id);
    assert_eq!(sent_entry.sender_id, requester_id);
    assert_eq!(sent_entry.recipient_id, provider_id);
    assert_eq!(sent_entry.direction, MailboxDirection::Sent);

    let response = request_with_session(
        Method::GET,
        "/mailbox?direction=inbox",
        &owner_session_cookie,
    )
    .await;
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, StatusCode::OK);
    let response_body: RestApiResponse<Vec<MailboxEntryDto>> =
        deserialize_json_body(body).await.unwrap();
    let inbox_entries = response_body.0.data.unwrap();
    let inbox_entry = inbox_entries
        .iter()
        .find(|entry| entry.booking_id == Some(booking.booking_id))
        .expect("provider mailbox should contain inbox booking request");
    assert_eq!(inbox_entry.owner_id, provider_id);
    assert_eq!(inbox_entry.sender_id, requester_id);
    assert_eq!(inbox_entry.recipient_id, provider_id);
    assert_eq!(inbox_entry.direction, MailboxDirection::Inbox);
    assert!(inbox_entry.read_at.is_none());

    let response = request_with_session(
        Method::POST,
        &format!("/mailbox/{}/read", inbox_entry.mailbox_entry_id),
        &owner_session_cookie,
    )
    .await;
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, StatusCode::OK);
    let response_body: RestApiResponse<MailboxEntryDto> =
        deserialize_json_body(body).await.unwrap();
    let read_entry = response_body.0.data.unwrap();
    assert!(read_entry.read_at.is_some());
}

#[tokio::test]
async fn test_article_owner_can_administer_booking_via_mine_routes() {
    let article = create_article()
        .await
        .expect("Failed to create article for owner booking test");
    let owner_session_cookie = login_and_get_session_cookie().await;
    let requester_session_cookie = signup_and_get_session_cookie().await;

    let booking = create_booking(
        article.article_id,
        &requester_session_cookie,
        &create_booking_payload(13, 16),
    )
    .await;

    let response = request_with_session(
        Method::GET,
        &format!("/article/mine/{}/bookings", article.article_id),
        &owner_session_cookie,
    )
    .await;
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, StatusCode::OK);
    let response_body: RestApiResponse<Vec<BookingDto>> =
        deserialize_json_body(body).await.unwrap();
    let bookings = response_body.0.data.unwrap();
    assert!(
        bookings
            .iter()
            .any(|item| item.booking_id == booking.booking_id)
    );

    let response = request_with_session(
        Method::POST,
        &format!(
            "/article/mine/{}/bookings/{}/confirm",
            article.article_id, booking.booking_id
        ),
        &owner_session_cookie,
    )
    .await;
    let (parts, body) = response.into_parts();
    let response_body: RestApiResponse<BookingDto> = deserialize_json_body(body).await.unwrap();
    assert_eq!(
        parts.status,
        StatusCode::OK,
        "unexpected owner confirm response: {:?}",
        response_body.0
    );
    let confirmed = response_body.0.data.unwrap();
    assert_eq!(confirmed.status, BookingStatus::Confirmed);

    let response = request_with_session(
        Method::POST,
        &format!(
            "/article/mine/{}/bookings/{}/cancel",
            article.article_id, booking.booking_id
        ),
        &requester_session_cookie,
    )
    .await;
    assert_eq!(response.into_parts().0.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_create_confirm_and_list_booking() {
    let article = create_article()
        .await
        .expect("Failed to create article for booking test");
    let owner_session_cookie = login_and_get_session_cookie().await;
    let requester_session_cookie = signup_and_get_session_cookie().await;
    let payload = create_booking_payload(8, 12);

    let booking = create_booking(article.article_id, &requester_session_cookie, &payload).await;
    assert_eq!(booking.article_id, article.article_id);
    assert_eq!(booking.status, BookingStatus::Requested);
    assert_eq!(booking.requester_email, payload.requester_email);

    let response = request_with_session(
        Method::POST,
        &format!(
            "/article/{}/bookings/{}/confirm",
            article.article_id, booking.booking_id
        ),
        &owner_session_cookie,
    )
    .await;
    let (parts, body) = response.into_parts();
    let response_body: RestApiResponse<BookingDto> = deserialize_json_body(body).await.unwrap();
    assert_eq!(
        parts.status,
        StatusCode::OK,
        "unexpected confirm response: {:?}",
        response_body.0
    );
    let confirmed = response_body.0.data.unwrap();
    assert_eq!(confirmed.status, BookingStatus::Confirmed);
    assert!(confirmed.approved_by.is_some());
    assert!(confirmed.approved_at.is_some());

    let response = request_with_session(
        Method::GET,
        &format!("/article/{}/bookings", article.article_id),
        &owner_session_cookie,
    )
    .await;
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<Vec<BookingDto>> =
        deserialize_json_body(body).await.unwrap();
    let bookings = response_body.0.data.unwrap();
    assert!(
        bookings
            .iter()
            .any(|item| item.booking_id == booking.booking_id)
    );
}

#[tokio::test]
async fn test_rejects_overlapping_confirmed_booking() {
    let article = create_article()
        .await
        .expect("Failed to create article for booking test");
    let owner_session_cookie = login_and_get_session_cookie().await;
    let requester_session_cookie = signup_and_get_session_cookie().await;

    let first = create_booking(
        article.article_id,
        &requester_session_cookie,
        &create_booking_payload(8, 12),
    )
    .await;
    let response = request_with_session(
        Method::POST,
        &format!(
            "/article/{}/bookings/{}/confirm",
            article.article_id, first.booking_id
        ),
        &owner_session_cookie,
    )
    .await;
    let (parts, body) = response.into_parts();
    let response_body: RestApiResponse<BookingDto> = deserialize_json_body(body).await.unwrap();
    assert_eq!(
        parts.status,
        StatusCode::OK,
        "unexpected confirm response: {:?}",
        response_body.0
    );

    let overlapping = create_booking(
        article.article_id,
        &requester_session_cookie,
        &create_booking_payload(10, 11),
    )
    .await;
    let response = request_with_session(
        Method::POST,
        &format!(
            "/article/{}/bookings/{}/confirm",
            article.article_id, overlapping.booking_id
        ),
        &owner_session_cookie,
    )
    .await;
    assert_eq!(response.into_parts().0.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_article_owner_cannot_request_own_article() {
    let article = create_article()
        .await
        .expect("Failed to create article for self booking test");
    let owner_session_cookie = login_and_get_session_cookie().await;
    let payload = create_booking_payload(13, 16);

    let response = request_with_session_body(
        Method::POST,
        &format!("/article/{}/bookings", article.article_id),
        &owner_session_cookie,
        &payload,
    )
    .await;

    assert_eq!(response.into_parts().0.status, StatusCode::BAD_REQUEST);
}
