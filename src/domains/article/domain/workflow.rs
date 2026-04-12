use std::sync::Arc;

use crate::{
    common::error::AppError,
    domains::{
        article::dto::article_dto::{
            CreateArticleDto, CreateArticleWithPicturesResponseDto, ExistingArticlePictureDto,
            NewArticlePictureDto, UpdateArticleDtoWithIdDto, UpdateArticleWithPicturesResponseDto,
        },
        file::dto::file_dto::FileDto,
    },
};
use deadpool_postgres::Pool;

#[async_trait::async_trait]
pub trait ArticleWorkflowServiceTrait: Send + Sync {
    fn create_service(
        pool: Pool,
        file_service: Arc<dyn crate::domains::file::FileServiceTrait>,
    ) -> Arc<dyn ArticleWorkflowServiceTrait>
    where
        Self: Sized;

    async fn create_article_with_pictures(
        &self,
        payload: CreateArticleDto,
        pictures: Vec<FileDto>,
        new_picture_meta: Vec<NewArticlePictureDto>,
    ) -> Result<CreateArticleWithPicturesResponseDto, AppError>;

    async fn update_article_with_pictures(
        &self,
        article_id: uuid::Uuid,
        payload: UpdateArticleDtoWithIdDto,
        existing_pictures: Vec<ExistingArticlePictureDto>,
        delete_file_ids: Vec<uuid::Uuid>,
        new_pictures: Vec<FileDto>,
        new_picture_meta: Vec<NewArticlePictureDto>,
    ) -> Result<UpdateArticleWithPicturesResponseDto, AppError>;
}
