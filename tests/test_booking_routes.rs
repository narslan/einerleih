use axum::http::{Method, StatusCode};
use chrono::{TimeZone, Utc};

use einerleih::{
    common::{dto::RestApiResponse, error::AppError},
    domains::{
        article::{
            ArticleStatus,
            dto::article_dto::{ArticleDto, CreateArticleDto},
        },
        booking::{
            BookingStatus,
            dto::booking_dto::{BookingDto, CreateBookingDto},
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
        name: format!("book-{short_id}"),
        category: uuid::Uuid::parse_str(TEST_CATEGORY_ID).unwrap(),
        description: "Artikel fuer Buchungs-Tests".to_string(),
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

fn create_booking_payload(start_hour: u32, end_hour: u32) -> CreateBookingDto {
    CreateBookingDto {
        requested_by: None,
        requester_name: Some("API Test".to_string()),
        requester_email: Some("apitest@example.com".to_string()),
        note: Some("Bitte reservieren".to_string()),
        start_time: Utc.with_ymd_and_hms(2026, 5, 2, start_hour, 0, 0).unwrap(),
        end_time: Utc.with_ymd_and_hms(2026, 5, 2, end_hour, 0, 0).unwrap(),
        created_by: uuid::Uuid::nil(),
        modified_by: uuid::Uuid::nil(),
    }
}

async fn create_booking(
    article_id: uuid::Uuid,
    token: &str,
    payload: &CreateBookingDto,
) -> BookingDto {
    let response = request_with_auth_body(
        Method::POST,
        &format!("/article/{article_id}/bookings"),
        token,
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
async fn test_create_confirm_and_list_booking() {
    let article = create_article()
        .await
        .expect("Failed to create article for booking test");
    let token = login_and_get_token().await;
    let payload = create_booking_payload(8, 12);

    let booking = create_booking(article.article_id, &token, &payload).await;
    assert_eq!(booking.article_id, article.article_id);
    assert_eq!(booking.status, BookingStatus::Requested);
    assert_eq!(booking.requester_email, payload.requester_email);

    let response = request_with_auth(
        Method::POST,
        &format!(
            "/article/{}/bookings/{}/confirm",
            article.article_id, booking.booking_id
        ),
        &token,
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

    let response = request_with_auth(
        Method::GET,
        &format!("/article/{}/bookings", article.article_id),
        &token,
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
    let token = login_and_get_token().await;

    let first = create_booking(article.article_id, &token, &create_booking_payload(8, 12)).await;
    let response = request_with_auth(
        Method::POST,
        &format!(
            "/article/{}/bookings/{}/confirm",
            article.article_id, first.booking_id
        ),
        &token,
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

    let overlapping =
        create_booking(article.article_id, &token, &create_booking_payload(10, 11)).await;
    let response = request_with_auth(
        Method::POST,
        &format!(
            "/article/{}/bookings/{}/confirm",
            article.article_id, overlapping.booking_id
        ),
        &token,
    )
    .await;
    assert_eq!(response.into_parts().0.status, StatusCode::BAD_REQUEST);
}
