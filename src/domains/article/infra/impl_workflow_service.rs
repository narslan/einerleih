use std::sync::Arc;

use crate::{
    common::error::AppError,
    domains::{
        article::{
            domain::{
                repository::ArticleRepository, workflow::ArticleWorkflowServiceTrait,
            },
            dto::article_dto::{
                ArticleDto, CreateArticleDto, CreateArticleWithPicturesResponseDto,
                ExistingArticlePictureDto, NewArticlePictureDto, UpdateArticleDtoWithIdDto,
                UpdateArticleWithPicturesResponseDto,
            },
            infra::impl_repository::ArticleRepo,
        },
        file::{
            FileDto, FileRepo, FileRepository, FileServiceTrait, UploadedFile,
            dto::file_dto::{UploadFileDto, UploadedFileDto},
        },
    },
};
use async_trait::async_trait;
use deadpool_postgres::Pool;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Clone)]
pub struct ArticleWorkflowService {
    pool: Pool,
    article_repo: Arc<dyn ArticleRepository + Send + Sync>,
    file_repo: Arc<dyn FileRepository + Send + Sync>,
    file_service: Arc<dyn FileServiceTrait>,
}

#[async_trait]
impl ArticleWorkflowServiceTrait for ArticleWorkflowService {
    fn create_service(
        pool: Pool,
        file_service: Arc<dyn FileServiceTrait>,
    ) -> Arc<dyn ArticleWorkflowServiceTrait> {
        Arc::new(Self {
            pool,
            article_repo: Arc::new(ArticleRepo {}),
            file_repo: Arc::new(FileRepo {}),
            file_service,
        })
    }

    async fn create_article_with_pictures(
        &self,
        payload: CreateArticleDto,
        pictures: Vec<FileDto>,
        new_picture_meta: Vec<NewArticlePictureDto>,
    ) -> Result<CreateArticleWithPicturesResponseDto, AppError> {
        let mut client = self.pool.get().await.map_err(AppError::DatabaseError)?;
        let mut tx = client.transaction().await.map_err(|err| {
            tracing::error!("Error opening article workflow transaction: {err}");
            AppError::InternalError
        })?;

        let article_id = Uuid::new_v4();
        let modified_by = payload.modified_by;
        self.article_repo
            .create(&mut tx, article_id, payload)
            .await
            .map_err(|err| {
                tracing::error!("Error creating article in workflow: {err}");
                AppError::DatabaseError(err)
            })?;

        let new_picture_meta = if new_picture_meta.is_empty() {
            pictures
                .iter()
                .enumerate()
                .map(|(index, _)| NewArticlePictureDto {
                    sort_order: index as i32,
                    is_cover: index == 0,
                })
                .collect()
        } else {
            new_picture_meta
        };

        if pictures.len() != new_picture_meta.len() {
            return Err(AppError::ValidationError(
                "new_picture_meta must contain one entry for each uploaded photo".into(),
            ));
        }

        let cover_count = new_picture_meta
            .iter()
            .filter(|picture| picture.is_cover)
            .count();
        if cover_count > 1 {
            return Err(AppError::ValidationError(
                "only one picture can be marked as cover".into(),
            ));
        }

        let mut uploaded_files: Vec<UploadedFileDto> = Vec::with_capacity(pictures.len());
        for (picture, picture_meta) in pictures.into_iter().zip(new_picture_meta.into_iter()) {
            let uploaded_file = self
                .file_service
                .process_article_picture_upload(
                    &mut tx,
                    &UploadFileDto {
                        file: picture,
                        article_id,
                        modified_by,
                        sort_order: picture_meta.sort_order,
                        is_cover: picture_meta.is_cover,
                    },
                )
                .await?;
            uploaded_files.push(uploaded_file);
        }

        tx.commit().await.map_err(|err| {
            tracing::error!("Error committing article workflow transaction: {err}");
            AppError::InternalError
        })?;

        let created_article = self
            .article_repo
            .find_by_id(self.pool.clone(), article_id)
            .await
            .map_err(|err| {
                tracing::error!("Error loading created article in workflow: {err}");
                AppError::DatabaseError(err)
            })?
            .ok_or_else(|| AppError::NotFound("Article not found".into()))?;
        let created_files = self
            .file_repo
            .find_by_article_id(self.pool.clone(), article_id)
            .await
            .map_err(|err| {
                tracing::error!("Error loading created article files in workflow: {err}");
                AppError::DatabaseError(err)
            })?;

        Ok(CreateArticleWithPicturesResponseDto {
            article: ArticleDto::from_article_with_pictures(created_article, created_files),
            uploaded_files,
        })
    }

    async fn update_article_with_pictures(
        &self,
        article_id: uuid::Uuid,
        payload: UpdateArticleDtoWithIdDto,
        existing_pictures: Vec<ExistingArticlePictureDto>,
        delete_file_ids: Vec<uuid::Uuid>,
        new_pictures: Vec<FileDto>,
        new_picture_meta: Vec<NewArticlePictureDto>,
    ) -> Result<UpdateArticleWithPicturesResponseDto, AppError> {
        let mut client = self.pool.get().await.map_err(AppError::DatabaseError)?;
        let mut tx = client.transaction().await.map_err(|err| {
            tracing::error!("Error opening article update workflow transaction: {err}");
            AppError::InternalError
        })?;

        self.article_repo
            .find_by_id(self.pool.clone(), article_id)
            .await
            .map_err(|err| {
                tracing::error!("Error loading article for update workflow: {err}");
                AppError::DatabaseError(err)
            })?
            .ok_or_else(|| AppError::NotFound("Article not found".into()))?;

        let existing_files = self
            .file_repo
            .find_by_article_id(self.pool.clone(), article_id)
            .await
            .map_err(|err| {
                tracing::error!("Error loading files for update workflow: {err}");
                AppError::DatabaseError(err)
            })?;

        let delete_file_ids_set: HashSet<Uuid> = delete_file_ids.iter().copied().collect();
        let article_file_ids_set: HashSet<Uuid> = existing_files.iter().map(|file| file.id).collect();

        if new_pictures.len() != new_picture_meta.len() {
            return Err(AppError::ValidationError(
                "new_picture_meta must contain one entry for each uploaded photo".into(),
            ));
        }

        if delete_file_ids_set
            .iter()
            .any(|file_id| !article_file_ids_set.contains(file_id))
        {
            return Err(AppError::ValidationError(
                "delete_file_ids contains a file that does not belong to the article".into(),
            ));
        }

        if existing_pictures
            .iter()
            .any(|picture| !article_file_ids_set.contains(&picture.id))
        {
            return Err(AppError::ValidationError(
                "existing_pictures contains a file that does not belong to the article".into(),
            ));
        }

        if existing_pictures
            .iter()
            .any(|picture| delete_file_ids_set.contains(&picture.id))
        {
            return Err(AppError::ValidationError(
                "a picture cannot be updated and deleted in the same request".into(),
            ));
        }

        if existing_pictures.len() + delete_file_ids_set.len() != existing_files.len() {
            return Err(AppError::ValidationError(
                "existing_pictures must describe all remaining files of the article".into(),
            ));
        }

        let final_cover_count = existing_pictures
            .iter()
            .filter(|picture| picture.is_cover)
            .count()
            + new_picture_meta
                .iter()
                .filter(|picture| picture.is_cover)
                .count();

        if final_cover_count > 1 {
            return Err(AppError::ValidationError(
                "only one picture can be marked as cover".into(),
            ));
        }

        let modified_by = payload.modified_by;
        let updated_article = self
            .article_repo
            .update(&mut tx, article_id, payload)
            .await
            .map_err(|err| {
                tracing::error!("Error updating article in workflow: {err}");
                AppError::DatabaseError(err)
            })?
            .ok_or_else(|| AppError::NotFound("Article not found".into()))?;

        for existing_picture in &existing_pictures {
            self.file_repo
                .update_file_metadata(
                    &mut tx,
                    existing_picture.id,
                    article_id,
                    existing_picture.sort_order,
                    existing_picture.is_cover,
                    modified_by,
                )
                .await
                .map_err(|err| {
                    tracing::error!("Error updating file metadata in workflow: {err}");
                    AppError::DatabaseError(err)
                })?
                .ok_or_else(|| AppError::NotFound("File not found".into()))?;
        }

        let mut deleted_files: Vec<UploadedFile> = Vec::new();
        for delete_file_id in &delete_file_ids {
            let file = existing_files
                .iter()
                .find(|file| &file.id == delete_file_id)
                .cloned()
                .ok_or_else(|| AppError::NotFound("File not found".into()))?;

            let deleted = self
                .file_repo
                .delete(&mut tx, *delete_file_id)
                .await
                .map_err(|err| {
                    tracing::error!("Error deleting file in workflow: {err}");
                    AppError::DatabaseError(err)
                })?;

            if !deleted {
                return Err(AppError::NotFound("File not found".into()));
            }

            deleted_files.push(file);
        }

        for (picture, picture_meta) in new_pictures.into_iter().zip(new_picture_meta.into_iter()) {
            self.file_service
                .process_article_picture_upload(
                    &mut tx,
                    &UploadFileDto {
                        file: picture,
                        article_id,
                        modified_by,
                        sort_order: picture_meta.sort_order,
                        is_cover: picture_meta.is_cover,
                    },
                )
                .await?;
        }

        tx.commit().await.map_err(|err| {
            tracing::error!("Error committing article update workflow transaction: {err}");
            AppError::InternalError
        })?;

        for deleted_file in deleted_files {
            self.file_service
                .remove_file_from_disk(deleted_file.file_relative_path.as_str())?;
        }

        let updated_files = self
            .file_repo
            .find_by_article_id(self.pool.clone(), article_id)
            .await
            .map_err(|err| {
                tracing::error!("Error loading updated files in workflow: {err}");
                AppError::DatabaseError(err)
            })?;

        Ok(UpdateArticleWithPicturesResponseDto {
            article: ArticleDto::from_article_with_pictures(
                updated_article,
                updated_files.clone(),
            ),
            uploaded_files: updated_files.into_iter().map(UploadedFileDto::from).collect(),
        })
    }
}
