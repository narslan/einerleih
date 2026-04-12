use crate::common::{config::Config, error::AppError};
use crate::domains::file::domain::model::UploadedFileType;
use crate::domains::file::domain::repository::FileRepository;
use crate::domains::file::domain::service::FileServiceTrait;
use crate::domains::file::dto::file_dto::{CreateFileDto, UploadFileDto, UploadedFileDto};
use crate::domains::file::infra::impl_repository::FileRepo;

use async_trait::async_trait;
use deadpool_postgres::{Pool, Transaction};

use std::path::{Path as FilePath, PathBuf};
use std::sync::Arc;
use uuid::Uuid;

/// Service struct for handling file-related operations
/// such as uploading, deleting, and fetching files.
/// It uses a repository pattern to abstract the data access layer.
#[derive(Clone)]
pub struct FileService {
    config: Config,
    pool: Pool,
    repo: Arc<dyn FileRepository + Send + Sync>,
}

/// Implementation of the FileService struct
#[async_trait]
impl FileServiceTrait for FileService {
    /// constructor for the service.
    fn create_service(config: Config, pool: Pool) -> Arc<dyn FileServiceTrait> {
        Arc::new(Self {
            config,
            pool,
            repo: Arc::new(FileRepo {}),
        })
    }

    /// Uploads a picture for an article.
    /// Validates the file, writes it to disk, and stores its metadata in the database.
    /// Returns the uploaded file's metadata.
    async fn process_article_picture_upload(
        &self,
        tx: &mut Transaction<'_>,
        upload_file_dto: &UploadFileDto,
    ) -> Result<UploadedFileDto, AppError> {
        let file_dto = &upload_file_dto.file;

        if file_dto.data.is_empty() {
            tracing::error!("File data is empty.");
            return Err(AppError::InvalidFileData);
        }

        if file_dto.data.len() > self.config.asset_max_size {
            tracing::error!("File size exceeded: {} bytes", file_dto.data.len());
            return Err(AppError::FileSizeExceeded);
        }

        let file_id = Uuid::new_v4();
        let (unique_filename, file_relative_path, file_path) =
            self.build_file_path(file_id, &file_dto.original_filename);

        self.write_file_to_disk(&file_path, &file_dto.data)?;

        let file_url = format!("/file/{file_id}");

        let create_file_dto = CreateFileDto {
            id: file_id,
            file_name: unique_filename,
            origin_file_name: file_dto.original_filename.clone(),
            file_relative_path,
            file_url,
            content_type: file_dto.content_type.clone(),
            file_size: file_dto.data.len() as i64,
            file_type: UploadedFileType::Foto,
            article_id: upload_file_dto.article_id,
            sort_order: upload_file_dto.sort_order,
            is_cover: upload_file_dto.is_cover,
            modified_by: upload_file_dto.modified_by,
        };

        let uploaded_file = self
            .repo
            .create_file(tx, create_file_dto)
            .await
            .map_err(|err| {
                tracing::error!("Error uploading file: {}", err);
                AppError::DatabaseError(err)
            })?;

        Ok(UploadedFileDto::from(uploaded_file))
    }

    /// Retrieves the metadata of a file by its id.
    async fn get_file_metadata(
        &self,
        file_id: uuid::Uuid,
    ) -> Result<Option<UploadedFileDto>, AppError> {
        let uploaded_file = self
            .repo
            .find_by_id(self.pool.clone(), file_id.clone())
            .await
            .map_err(|err| {
                tracing::error!("Error retrieving file: {}", err);
                AppError::DatabaseError(err)
            });

        match uploaded_file {
            Ok(Some(file)) => Ok(Some(UploadedFileDto::from(file))),
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }

    fn remove_file_from_disk(&self, file_relative_path: &str) -> Result<(), AppError> {
        let file_path =
            FilePath::new(self.config.assets_private_path.as_str()).join(file_relative_path);

        if !file_path.exists() {
            return Ok(());
        }

        std::fs::remove_file(&file_path).map_err(|err| {
            tracing::error!("Error deleting file from filesystem: {}", err);
            AppError::InternalError
        })
    }

    /// Deletes a file by its id.
    /// Removes the file from the filesystem and deletes its metadata from the database.
    /// Returns a success message if the deletion was successful.
    async fn delete_file(&self, file_id: uuid::Uuid) -> Result<String, AppError> {
        let mut client = self.pool.get().await.map_err(AppError::DatabaseError)?;
        let mut tx = client.transaction().await.map_err(|err| {
            tracing::error!("Error opening file transaction: {}", err);
            AppError::InternalError
        })?;

        let to_delete_file = self
            .repo
            .find_by_id(self.pool.clone(), file_id)
            .await
            .map_err(|err| {
                tracing::error!("Error retrieving file: {}", err);
                AppError::DatabaseError(err)
            })?;

        if to_delete_file.is_none() {
            return Err(AppError::NotFound("File not found".into()));
        }

        let deletion_result = self.repo.delete(&mut tx, file_id).await.map_err(|err| {
            tracing::error!("Error deleting file: {}", err);
            AppError::DatabaseError(err)
        })?;

        if !deletion_result {
            return Err(AppError::NotFound("File not found".into()));
        }

        tx.commit().await.map_err(|err| {
            tracing::error!("Error committing file deletion: {}", err);
            AppError::InternalError
        })?;

        self.remove_file_from_disk(to_delete_file.as_ref().unwrap().file_relative_path.as_str())?;

        Ok("File deleted successfully".into())
    }
}
/// Internal helper methods defined on `FileService`.
impl FileService {
    /// Ensures the generated filename is unique within the given directory.
    fn generate_storage_filename(file_id: Uuid, original: &str) -> String {
        let path = FilePath::new(original);
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|value| value.trim())
            .filter(|value| !value.is_empty());

        match ext {
            Some(ext) => format!("{file_id}.{ext}"),
            None => file_id.to_string(),
        }
    }

    /// Constructs a unique filename, relative path, and absolute disk path for the upload.
    fn build_file_path(&self, file_id: Uuid, original_filename: &str) -> (String, String, PathBuf) {
        let base_dir = self.config.assets_private_path.as_str();
        let base_dir_with_profile =
            FilePath::new(base_dir).join(UploadedFileType::Foto.to_string());

        let unique_filename = FileService::generate_storage_filename(file_id, original_filename);
        let file_path = base_dir_with_profile.join(&unique_filename);

        let relative_path = format!("{}/{}", UploadedFileType::Foto, unique_filename);
        (unique_filename, relative_path, file_path)
    }

    /// Writes the file's byte data to the disk, creating directories as needed.
    fn write_file_to_disk(&self, file_path: &FilePath, data: &[u8]) -> Result<(), AppError> {
        let parent = file_path.parent().ok_or(AppError::InternalError)?;
        std::fs::create_dir_all(parent).map_err(|err| {
            tracing::error!("Error creating directory: {}", err);
            AppError::InternalError
        })?;

        std::fs::write(file_path, data).map_err(|err| {
            tracing::error!("Error writing file: {}", err);
            AppError::InternalError
        })?;

        if !file_path.exists() {
            tracing::error!("File was not written successfully.");
            return Err(AppError::InternalError);
        }

        Ok(())
    }
}
