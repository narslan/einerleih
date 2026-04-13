// This module defines the `ArticleRepository` trait, which abstracts
// the database operations related to article management.

use crate::domains::article::dto::article_dto::{
    ArticleRelationDto, ArticleTagDto, CreateArticleDto, NormalizedArticleTag,
    UpdateArticleDtoWithIdDto,
};

use super::model::Article;
use async_trait::async_trait;
use deadpool_postgres::{Pool, PoolError, Transaction};
use uuid::Uuid;

/// Trait representing repository-level operations for article entities.
/// Provides an interface for data persistence and retrieval of article records.
#[async_trait]
pub trait ArticleRepository: Send + Sync {
    /// Retrieves all articles from the database.
    async fn find_all(&self, pool: Pool) -> Result<Vec<Article>, PoolError>;

    /// Finds a article by its unique identifier.
    async fn find_by_id(&self, pool: Pool, id: Uuid) -> Result<Option<Article>, PoolError>;

    /// Retrieves articles owned by the given user.
    async fn find_by_owner(&self, pool: Pool, owner_id: Uuid) -> Result<Vec<Article>, PoolError>;

    /// Returns available categories for article assignment.
    async fn find_categories(&self, pool: Pool) -> Result<Vec<ArticleRelationDto>, PoolError>;

    /// Returns available towns for article assignment.
    async fn find_towns(&self, pool: Pool) -> Result<Vec<ArticleRelationDto>, PoolError>;

    /// Retrieves user-defined tags assigned to the given article.
    async fn find_tags_by_article_id(
        &self,
        pool: Pool,
        article_id: Uuid,
    ) -> Result<Vec<ArticleTagDto>, PoolError>;

    /// Replaces all tags assigned to an article within the given transaction.
    async fn replace_tags(
        &self,
        tx: &mut Transaction<'_>,
        article_id: Uuid,
        tags: &[NormalizedArticleTag],
        actor_id: Uuid,
    ) -> Result<(), PoolError>;

    /// Creates a new article record in the database within the given transaction.
    async fn create(
        &self,
        tx: &mut Transaction<'_>,
        id: Uuid,
        article: CreateArticleDto,
    ) -> Result<(), PoolError>;

    /// Updates an existing article record with new data.
    async fn update(
        &self,
        tx: &mut Transaction<'_>,
        id: Uuid,
        article: UpdateArticleDtoWithIdDto,
    ) -> Result<Option<Article>, PoolError>;

    /// Deletes a article record by its ID.
    async fn delete(&self, tx: &mut Transaction<'_>, id: Uuid) -> Result<bool, PoolError>;
}
