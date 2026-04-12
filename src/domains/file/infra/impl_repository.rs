use crate::domains::file::{
    domain::{model::UploadedFile, model::UploadedFileType, repository::FileRepository},
    dto::file_dto::CreateFileDto,
};
use async_trait::async_trait;
use deadpool_postgres::{Pool, PoolError, Transaction};
use tokio_postgres::Row;
use uuid::Uuid;

pub struct FileRepo;

const CREATE_FILE_QUERY: &str = r#"
    INSERT INTO uploaded_files
        (
            id,
            file_name,
            origin_file_name,
            file_relative_path,
            file_url,
            content_type,
            file_size,
            file_type,
            article_id,
            sort_order,
            is_cover,
            created_by,
            modified_by
        )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
    RETURNING
        id,
        file_name,
        origin_file_name,
        file_relative_path,
        file_url,
        content_type,
        file_size,
        file_type,
        article_id,
        sort_order,
        is_cover,
        created_by,
        created_at,
        modified_by,
        modified_at
"#;

const FIND_FILE_BY_ID_QUERY: &str = r#"
    SELECT
        id,
        file_name,
        origin_file_name,
        file_relative_path,
        file_url,
        content_type,
        file_size,
        file_type,
        article_id,
        sort_order,
        is_cover,
        created_by,
        created_at,
        modified_by,
        modified_at
    FROM uploaded_files
    WHERE id = $1
"#;

const FIND_FILES_BY_ARTICLE_ID_QUERY: &str = r#"
    SELECT
        id,
        file_name,
        origin_file_name,
        file_relative_path,
        file_url,
        content_type,
        file_size,
        file_type,
        article_id,
        sort_order,
        is_cover,
        created_by,
        created_at,
        modified_by,
        modified_at
    FROM uploaded_files
    WHERE article_id = $1
    ORDER BY sort_order ASC, created_at ASC, id ASC
"#;

const UPDATE_FILE_METADATA_QUERY: &str = r#"
    UPDATE uploaded_files
    SET
        article_id = $2,
        sort_order = $3,
        is_cover = $4,
        modified_by = $5,
        modified_at = NOW()
    WHERE id = $1
    RETURNING
        id,
        file_name,
        origin_file_name,
        file_relative_path,
        file_url,
        content_type,
        file_size,
        file_type,
        article_id,
        sort_order,
        is_cover,
        created_by,
        created_at,
        modified_by,
        modified_at
"#;

fn map_uploaded_file_row(row: &Row) -> UploadedFile {
    let id: Uuid = row.get(0);
    let file_type: String = row.get(7);
    UploadedFile {
        id,
        file_name: row.get(1),
        origin_file_name: row.get(2),
        file_relative_path: row.get(3),
        file_url: format!("/file/{id}"),
        content_type: row.get(5),
        file_size: row.get(6),
        file_type: UploadedFileType::from_db(&file_type)
            .unwrap_or_else(|| panic!("invalid file type in database: {file_type}")),
        article_id: row.get(8),
        sort_order: row.get(9),
        is_cover: row.get(10),
        created_by: row.get(11),
        created_at: row.get(12),
        modified_by: row.get(13),
        modified_at: row.get(14),
    }
}

#[async_trait]
impl FileRepository for FileRepo {
    async fn create_file(
        &self,
        tx: &mut Transaction<'_>,
        file: CreateFileDto,
    ) -> Result<UploadedFile, PoolError> {
        let stmt = tx.prepare_cached(CREATE_FILE_QUERY).await?;
        let row = tx
            .query_one(
                &stmt,
                &[
                    &file.id,
                    &file.file_name,
                    &file.origin_file_name,
                    &file.file_relative_path,
                    &file.file_url,
                    &file.content_type,
                    &file.file_size,
                    &file.file_type.as_str(),
                    &file.article_id,
                    &file.sort_order,
                    &file.is_cover,
                    &file.modified_by,
                    &file.modified_by,
                ],
            )
            .await?;

        Ok(map_uploaded_file_row(&row))
    }

    async fn find_by_id(
        &self,
        pool: Pool,
        id: uuid::Uuid,
    ) -> Result<Option<UploadedFile>, PoolError> {
        let client = pool.get().await?;
        let stmt = client.prepare_cached(FIND_FILE_BY_ID_QUERY).await?;
        let row = client.query_opt(&stmt, &[&id]).await?;
        Ok(row.map(|row| map_uploaded_file_row(&row)))
    }

    async fn find_by_article_id(
        &self,
        pool: Pool,
        article_id: uuid::Uuid,
    ) -> Result<Vec<UploadedFile>, PoolError> {
        let client = pool.get().await?;
        let stmt = client
            .prepare_cached(FIND_FILES_BY_ARTICLE_ID_QUERY)
            .await?;
        let rows = client.query(&stmt, &[&article_id]).await?;
        Ok(rows.iter().map(map_uploaded_file_row).collect())
    }

    async fn update_file_metadata(
        &self,
        tx: &mut Transaction<'_>,
        file_id: uuid::Uuid,
        article_id: uuid::Uuid,
        sort_order: i32,
        is_cover: bool,
        modified_by: uuid::Uuid,
    ) -> Result<Option<UploadedFile>, PoolError> {
        let stmt = tx.prepare_cached(UPDATE_FILE_METADATA_QUERY).await?;
        let row = tx
            .query_opt(
                &stmt,
                &[&file_id, &article_id, &sort_order, &is_cover, &modified_by],
            )
            .await?;
        Ok(row.map(|row| map_uploaded_file_row(&row)))
    }

    async fn delete(&self, tx: &mut Transaction<'_>, id: uuid::Uuid) -> Result<bool, PoolError> {
        let stmt = tx
            .prepare_cached(r#"DELETE FROM uploaded_files WHERE id = $1"#)
            .await?;
        let affected_rows = tx.execute(&stmt, &[&id]).await?;
        Ok(affected_rows > 0)
    }
}
