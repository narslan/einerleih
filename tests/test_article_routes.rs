use axum::http::{Method, StatusCode};
use http_body_util::BodyExt;

use einerleih::{
    common::{dto::RestApiResponse, error::AppError},
    domains::article::{
        ArticleStatus,
        dto::article_dto::{
            ArticleDto, CreateArticleDto, CreateArticleWithPicturesResponseDto,
            UpdateArticleWithPicturesResponseDto,
        },
    },
};

mod test_helpers;

use test_helpers::{
    TEST_AUTH_USER_ID, TEST_CATEGORY_ID, TEST_TOWN_ID, deserialize_json_body, login_and_get_token,
    request, request_with_auth, request_with_auth_body, request_with_auth_raw,
};

async fn create_article() -> Result<(CreateArticleDto, ArticleDto), AppError> {
    let uuid_text = uuid::Uuid::new_v4().simple().to_string();
    let short_id = &uuid_text[..8];
    let payload = CreateArticleDto {
        name: format!("art-{short_id}"),
        category: uuid::Uuid::parse_str(TEST_CATEGORY_ID).unwrap(),
        description: "Ein Testartikel fuer die API-Tests".to_string(),
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
    let article_dto = response_body.0.data.unwrap();

    Ok((payload, article_dto))
}

fn build_multipart_article_request_with_photos(
    photos: &[(&str, &str, &[u8])],
    new_picture_meta: Option<serde_json::Value>,
) -> (String, Vec<u8>) {
    let boundary = format!("boundary-{}", uuid::Uuid::new_v4().simple());
    let uuid_text = uuid::Uuid::new_v4().simple().to_string();
    let short_id = &uuid_text[..8];
    let fields = [
        ("name", format!("art-{short_id}")),
        ("category", TEST_CATEGORY_ID.to_string()),
        ("description", "Artikel mit Bild".to_string()),
        ("town", TEST_TOWN_ID.to_string()),
        ("status", "aktiv".to_string()),
    ];

    let mut body = Vec::new();
    for (name, value) in fields {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
        );
        body.extend_from_slice(value.as_bytes());
        body.extend_from_slice(b"\r\n");
    }

    if let Some(new_picture_meta) = new_picture_meta {
        append_multipart_text_part(
            &mut body,
            &boundary,
            "new_picture_meta",
            &new_picture_meta.to_string(),
        );
    }

    for (filename, content_type, bytes) in photos {
        append_multipart_file_part(
            &mut body,
            &boundary,
            "photos",
            filename,
            content_type,
            bytes,
        );
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

    (boundary, body)
}

fn append_multipart_text_part(body: &mut Vec<u8>, boundary: &str, name: &str, value: &str) {
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
    );
    body.extend_from_slice(value.as_bytes());
    body.extend_from_slice(b"\r\n");
}

fn append_multipart_file_part(
    body: &mut Vec<u8>,
    boundary: &str,
    name: &str,
    filename: &str,
    content_type: &str,
    bytes: &[u8],
) {
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\n")
            .as_bytes(),
    );
    body.extend_from_slice(format!("Content-Type: {content_type}\r\n\r\n").as_bytes());
    body.extend_from_slice(bytes);
    body.extend_from_slice(b"\r\n");
}

async fn create_article_with_pictures(
    photos: &[(&str, &str, &[u8])],
    new_picture_meta: Option<serde_json::Value>,
) -> CreateArticleWithPicturesResponseDto {
    let token = login_and_get_token().await;
    let (boundary, payload) = build_multipart_article_request_with_photos(photos, new_picture_meta);

    let response = request_with_auth_raw(
        Method::POST,
        "/article/upload",
        &token,
        &format!("multipart/form-data; boundary={boundary}"),
        payload,
    )
    .await;
    let (parts, body) = response.into_parts();
    let response_body: RestApiResponse<CreateArticleWithPicturesResponseDto> =
        deserialize_json_body(body).await.unwrap();
    if parts.status != StatusCode::OK {
        panic!(
            "expected 200 from POST /article/upload, got {} with body {:?}",
            parts.status, response_body.0
        );
    }

    response_body.0.data.unwrap()
}

async fn create_article_with_picture() -> CreateArticleWithPicturesResponseDto {
    create_article_with_pictures(&[("artikel.png", "image/png", b"fake-image-data")], None).await
}

#[tokio::test]
async fn test_create_article() {
    let (payload, article_dto) = create_article().await.expect("Failed to create article");

    assert_ne!(article_dto.article_id, uuid::Uuid::nil());
    assert_eq!(article_dto.name, payload.name);
    assert_eq!(article_dto.category.id, payload.category);
    assert_eq!(article_dto.category.name, "Test Category");
    assert_eq!(article_dto.description, payload.description);
    assert_eq!(article_dto.town.id, payload.town);
    assert_eq!(article_dto.town.name, "Test Town");
    assert_eq!(article_dto.status, ArticleStatus::Aktiv);
    assert!(article_dto.cover_image.is_none());
    assert!(article_dto.pictures.is_empty());
    assert_eq!(
        article_dto.created_by,
        Some(uuid::Uuid::parse_str(TEST_AUTH_USER_ID).unwrap())
    );
    assert_eq!(
        article_dto.modified_by,
        Some(uuid::Uuid::parse_str(TEST_AUTH_USER_ID).unwrap())
    );
}

#[tokio::test]
async fn test_get_article_by_id() {
    let (_, created_article) = create_article().await.expect("Failed to create article");

    let token = login_and_get_token().await;
    let response = request_with_auth(
        Method::GET,
        &format!("/article/{}", created_article.article_id),
        &token,
    )
    .await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<ArticleDto> = deserialize_json_body(body).await.unwrap();
    let article_dto = response_body.0.data.unwrap();

    assert_eq!(article_dto.article_id, created_article.article_id);
    assert_eq!(article_dto.name, created_article.name);
    assert_eq!(article_dto.category.id, created_article.category.id);
    assert_eq!(article_dto.category.name, "Test Category");
    assert_eq!(article_dto.town.id, created_article.town.id);
    assert_eq!(article_dto.town.name, "Test Town");
    assert_eq!(article_dto.status, created_article.status);
}

#[tokio::test]
async fn test_get_articles_contains_created_article() {
    let (_, created_article) = create_article().await.expect("Failed to create article");

    let response = request(Method::GET, "/article").await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<Vec<ArticleDto>> = deserialize_json_body(body).await.unwrap();
    let articles = response_body.0.data.unwrap();

    assert!(
        articles
            .iter()
            .any(|article| article.article_id == created_article.article_id),
        "created article was not returned by GET /article"
    );
}

#[tokio::test]
async fn test_get_article_form_options() {
    let token = login_and_get_token().await;

    let response = request_with_auth(Method::GET, "/article/categories", &token).await;
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, StatusCode::OK);
    let response_body: RestApiResponse<Vec<einerleih::domains::article::dto::article_dto::ArticleRelationDto>> =
        deserialize_json_body(body).await.unwrap();
    let categories = response_body.0.data.unwrap();
    assert!(categories.iter().any(|item| item.id.to_string() == TEST_CATEGORY_ID));
    assert!(categories.iter().any(|item| item.name == "Test Category"));

    let response = request_with_auth(Method::GET, "/article/towns", &token).await;
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, StatusCode::OK);
    let response_body: RestApiResponse<Vec<einerleih::domains::article::dto::article_dto::ArticleRelationDto>> =
        deserialize_json_body(body).await.unwrap();
    let towns = response_body.0.data.unwrap();
    assert!(towns.iter().any(|item| item.id.to_string() == TEST_TOWN_ID));
    assert!(towns.iter().any(|item| item.name == "Test Town"));
}

#[tokio::test]
async fn test_get_article_by_id_public() {
    let (_, created_article) = create_article().await.expect("Failed to create article");

    let response = request(Method::GET, &format!("/article/{}", created_article.article_id)).await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<ArticleDto> = deserialize_json_body(body).await.unwrap();
    let article_dto = response_body.0.data.unwrap();
    assert_eq!(article_dto.article_id, created_article.article_id);
}

#[tokio::test]
async fn test_create_article_with_picture_and_fetch_file() {
    let created = create_article_with_picture().await;

    assert_eq!(created.article.status, ArticleStatus::Aktiv);
    assert_eq!(created.article.category.name, "Test Category");
    assert_eq!(created.article.town.name, "Test Town");
    assert_eq!(created.uploaded_files.len(), 1);
    assert_eq!(created.article.pictures.len(), 1);
    assert!(created.article.cover_image.is_some());

    let uploaded_file = &created.uploaded_files[0];
    assert_eq!(uploaded_file.article_id, Some(created.article.article_id));
    assert_eq!(uploaded_file.sort_order, 0);
    assert!(uploaded_file.is_cover);
    assert_eq!(uploaded_file.content_type, "image/png");
    assert_eq!(uploaded_file.file_url, format!("/file/{}", uploaded_file.id));
    assert_eq!(
        created.article.cover_image.as_ref().unwrap().file_url,
        format!("/file/{}", uploaded_file.id)
    );
    assert_eq!(created.article.cover_image.as_ref().unwrap().id, uploaded_file.id);
    assert_eq!(created.article.pictures[0].id, uploaded_file.id);
    assert_eq!(
        created.article.pictures[0].file_url,
        format!("/file/{}", uploaded_file.id)
    );

    let response = request(Method::GET, &format!("/file/{}", uploaded_file.id)).await;
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, StatusCode::OK);
    assert_eq!(
        parts.headers.get("content-type").unwrap(),
        "image/png"
    );

    let bytes = body.collect().await.unwrap().to_bytes();
    assert_eq!(&bytes[..], b"fake-image-data");
    assert_eq!(
        uploaded_file.created_by,
        Some(uuid::Uuid::parse_str(TEST_AUTH_USER_ID).unwrap())
    );
}

#[tokio::test]
async fn test_update_article_with_picture_changes() {
    let token = login_and_get_token().await;
    let created = create_article_with_pictures(&[
        ("titelbild.png", "image/png", b"first-image-data"),
        ("detailbild.jpg", "image/jpeg", b"second-image-data"),
    ], None)
    .await;
    let deleted_file = created.uploaded_files[0].clone();
    let original_file = created.uploaded_files[1].clone();

    let boundary = format!("boundary-{}", uuid::Uuid::new_v4().simple());
    let existing_pictures = serde_json::json!([
        {
            "id": original_file.id,
            "sort_order": 2,
            "is_cover": false
        }
    ]);
    let delete_file_ids = serde_json::json!([deleted_file.id]);
    let new_picture_meta = serde_json::json!([
        {
            "sort_order": 0,
            "is_cover": true
        }
    ]);

    let mut body = Vec::new();
    append_multipart_text_part(&mut body, &boundary, "name", "artikel-updated");
    append_multipart_text_part(&mut body, &boundary, "category", TEST_CATEGORY_ID);
    append_multipart_text_part(
        &mut body,
        &boundary,
        "description",
        "Artikel nach Bildaenderung",
    );
    append_multipart_text_part(&mut body, &boundary, "town", TEST_TOWN_ID);
    append_multipart_text_part(&mut body, &boundary, "status", "aktiv");
    append_multipart_text_part(
        &mut body,
        &boundary,
        "existing_pictures",
        &existing_pictures.to_string(),
    );
    append_multipart_text_part(
        &mut body,
        &boundary,
        "delete_file_ids",
        &delete_file_ids.to_string(),
    );
    append_multipart_text_part(
        &mut body,
        &boundary,
        "new_picture_meta",
        &new_picture_meta.to_string(),
    );
    append_multipart_file_part(
        &mut body,
        &boundary,
        "photos",
        "neu.webp",
        "image/webp",
        b"new-image-data",
    );
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

    let response = request_with_auth_raw(
        Method::PUT,
        &format!("/article/{}/upload", created.article.article_id),
        &token,
        &format!("multipart/form-data; boundary={boundary}"),
        body,
    )
    .await;
    let (parts, body) = response.into_parts();
    let response_body: RestApiResponse<UpdateArticleWithPicturesResponseDto> =
        deserialize_json_body(body).await.unwrap();
    if parts.status != StatusCode::OK {
        panic!(
            "expected 200 from PUT /article/{{id}}/upload, got {} with body {:?}",
            parts.status, response_body.0
        );
    }
    let updated = response_body.0.data.unwrap();

    assert_eq!(updated.article.article_id, created.article.article_id);
    assert_eq!(updated.article.name, "artikel-updated");
    assert_eq!(updated.article.description, "Artikel nach Bildaenderung");
    assert_eq!(updated.article.category.name, "Test Category");
    assert_eq!(updated.article.town.name, "Test Town");
    assert_eq!(updated.uploaded_files.len(), 2);
    assert_eq!(updated.article.pictures.len(), 2);

    let updated_original = updated
        .article
        .pictures
        .iter()
        .find(|file| file.id == original_file.id)
        .expect("existing file should still be present");
    assert_eq!(updated_original.sort_order, 2);
    assert!(!updated_original.is_cover);

    let new_file = updated
        .article
        .pictures
        .iter()
        .find(|file| file.id != original_file.id)
        .expect("new file should be present");
    assert_eq!(new_file.sort_order, 0);
    assert!(new_file.is_cover);
    assert_eq!(new_file.content_type, "image/webp");
    assert_eq!(updated.article.cover_image.as_ref().unwrap().id, new_file.id);
    assert!(
        updated
            .article
            .pictures
            .iter()
            .all(|file| file.id != deleted_file.id),
        "deleted file should not be present anymore"
    );

    let response = request_with_auth(Method::GET, &format!("/file/{}", new_file.id), &token).await;
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, StatusCode::OK);
    assert_eq!(parts.headers.get("content-type").unwrap(), "image/webp");
    let bytes = body.collect().await.unwrap().to_bytes();
    assert_eq!(&bytes[..], b"new-image-data");
}

#[tokio::test]
async fn test_create_article_with_picture_meta() {
    let created = create_article_with_pictures(
        &[
            ("erstes.png", "image/png", b"first-image-data"),
            ("zweites.webp", "image/webp", b"second-image-data"),
        ],
        Some(serde_json::json!([
            {
                "sort_order": 9,
                "is_cover": false
            },
            {
                "sort_order": 3,
                "is_cover": true
            }
        ])),
    )
    .await;

    assert_eq!(created.uploaded_files.len(), 2);
    assert_eq!(created.article.pictures.len(), 2);
    assert!(created.article.cover_image.is_some());

    let first_uploaded = &created.uploaded_files[0];
    let second_uploaded = &created.uploaded_files[1];

    assert_eq!(first_uploaded.sort_order, 9);
    assert!(!first_uploaded.is_cover);
    assert_eq!(second_uploaded.sort_order, 3);
    assert!(second_uploaded.is_cover);

    assert_eq!(
        created.article.cover_image.as_ref().unwrap().id,
        second_uploaded.id
    );
    assert_eq!(created.article.pictures[0].id, second_uploaded.id);
    assert_eq!(created.article.pictures[0].sort_order, 3);
    assert!(created.article.pictures[0].is_cover);
    assert_eq!(created.article.pictures[1].id, first_uploaded.id);
    assert_eq!(created.article.pictures[1].sort_order, 9);
    assert!(!created.article.pictures[1].is_cover);
}
