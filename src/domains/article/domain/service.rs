//! This module defines the `ArticleServiceTrait` which encapsulates the business logic
//! for managing articles in the system.

use std::sync::Arc;

use deadpool_postgres::Pool;

use crate::{
    common::error::AppError,
    domains::article::dto::article_dto::{
        ArticleDto, ArticleRelationDto, CreateArticleDto, UpdateArticleDtoWithIdDto,
    },
};

#[async_trait::async_trait]
/// Trait defining the contract for article-related business operations.
/// This includes creating, retrieving, updating, and deleting articles,

pub trait ArticleServiceTrait: Send + Sync {
    /// constructor for the service.
    fn create_service(pool: Pool) -> Arc<dyn ArticleServiceTrait>
    where
        Self: Sized;

    /// Retrieves a article by its unique ID.
    async fn get_article_by_id(&self, id: uuid::Uuid) -> Result<ArticleDto, AppError>;

    /// Retrieves a list of all articles.
    async fn get_articles(&self) -> Result<Vec<ArticleDto>, AppError>;

    /// Retrieves selectable categories for article forms.
    async fn get_categories(&self) -> Result<Vec<ArticleRelationDto>, AppError>;

    /// Retrieves selectable towns for article forms.
    async fn get_towns(&self) -> Result<Vec<ArticleRelationDto>, AppError>;

    /// Creates a new article from the provided payload.
    async fn create_article(&self, payload: CreateArticleDto) -> Result<ArticleDto, AppError>;

    /// Updates an existing article with new data.
    async fn update_article(
        &self,
        id: uuid::Uuid,
        payload: UpdateArticleDtoWithIdDto,
    ) -> Result<ArticleDto, AppError>;

    /// Deletes a article by its ID.
    async fn delete_article(&self, id: uuid::Uuid) -> Result<String, AppError>;
}
