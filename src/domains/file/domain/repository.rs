//! This module defines the `FileRepository` trait, which provides
//! an abstraction over database operations for managing uploaded files.

use crate::domains::file::dto::file_dto::CreateFileDto;

use super::model::UploadedFile;

use async_trait::async_trait;
use deadpool_postgres::{Pool, PoolError, Transaction};

#[async_trait]
/// Trait representing repository-level operations for uploaded file metadata.
/// Enables persistence, retrieval, and deletion of file records through database interactions.
pub trait FileRepository: Send + Sync {
    /// Inserts a new file record into the database using a transaction.
    async fn create_file(
        &self,
        tx: &mut Transaction<'_>,
        file: CreateFileDto,
    ) -> Result<UploadedFile, PoolError>;

    /// Finds a file record by its unique identifier.
    async fn find_by_id(
        &self,
        pool: Pool,
        id: uuid::Uuid,
    ) -> Result<Option<UploadedFile>, PoolError>;

    /// Returns all files associated with the given article ordered for display.
    async fn find_by_article_id(
        &self,
        pool: Pool,
        article_id: uuid::Uuid,
    ) -> Result<Vec<UploadedFile>, PoolError>;

    /// Updates the article assignment and display metadata of a stored file.
    async fn update_file_metadata(
        &self,
        tx: &mut Transaction<'_>,
        file_id: uuid::Uuid,
        article_id: uuid::Uuid,
        sort_order: i32,
        is_cover: bool,
        modified_by: uuid::Uuid,
    ) -> Result<Option<UploadedFile>, PoolError>;

    /// Deletes a file record by its unique identifier using a transaction.
    async fn delete(
        &self,
        tx: &mut Transaction<'_>,
        id: uuid::Uuid,
    ) -> Result<bool, PoolError>;
}
