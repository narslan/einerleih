use crate::{
    common::error::AppError,
    domains::{
        article::{
            domain::{repository::ArticleRepository, service::ArticleServiceTrait},
            dto::article_dto::{
                ArticleDto, ArticleRelationDto, CreateArticleDto, UpdateArticleDtoWithIdDto,
                normalize_article_tags,
            },
            infra::impl_repository::ArticleRepo,
        },
        file::{FileRepo, FileRepository},
    },
};
use async_trait::async_trait;
use deadpool_postgres::Pool;

use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;

/// Service struct for handling article-related operations
/// such as creating, updating, deleting, and fetching articles.
/// It uses a repository pattern to abstract the data access layer.
#[derive(Clone)]
pub struct ArticleService {
    pub pool: Pool,
    pub repo: Arc<dyn ArticleRepository + Send + Sync>,
    pub file_repo: Arc<dyn FileRepository + Send + Sync>,
}

#[async_trait]
impl ArticleServiceTrait for ArticleService {
    /// constructor for the service.
    fn create_service(pool: Pool) -> Arc<dyn ArticleServiceTrait> {
        Arc::new(Self {
            pool,
            repo: Arc::new(ArticleRepo {}),
            file_repo: Arc::new(FileRepo {}),
        })
    }

    /// Retrieves a article by their ID.
    async fn get_article_by_id(&self, id: Uuid) -> Result<ArticleDto, AppError> {
        match self.repo.find_by_id(self.pool.clone(), id).await {
            Ok(Some(article)) => {
                let pictures = self
                    .file_repo
                    .find_by_article_id(self.pool.clone(), id)
                    .await
                    .map_err(AppError::DatabaseError)?;
                let tags = self
                    .repo
                    .find_tags_by_article_id(self.pool.clone(), id)
                    .await
                    .map_err(AppError::DatabaseError)?;
                Ok(ArticleDto::from_article_with_pictures(
                    article, pictures, tags,
                ))
            }
            Ok(None) => Err(AppError::NotFound("Article not found".into())),
            Err(err) => {
                tracing::error!("Error retrieving article: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    /*   /// Retrieves article list by condition
    /// Returns a vector of ArticleDto objects.
    async fn get_articles(
        &self,
        search_article_dto: SearchArticleDto,
    ) -> Result<Vec<ArticleDto>, AppError> {
        match self
            .repo
            .find_all(self.pool.clone())
            .await
        {
            Ok(articles) => {
                let article_dtos: Vec<ArticleDto> = articles.into_iter().map(Into::into).collect();
                Ok(article_dtos)
            }
            Err(err) => {
                tracing::error!("Error fetching articles: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }*/

    /// Retrieves all articles.
    /// Returns a vector of ArticleDto objects.
    async fn get_articles(&self) -> Result<Vec<ArticleDto>, AppError> {
        match self.repo.find_all(self.pool.clone()).await {
            Ok(articles) => {
                let mut article_dtos: Vec<ArticleDto> = Vec::with_capacity(articles.len());
                for article in articles {
                    let pictures = self
                        .file_repo
                        .find_by_article_id(self.pool.clone(), article.article_id)
                        .await
                        .map_err(AppError::DatabaseError)?;
                    let tags = self
                        .repo
                        .find_tags_by_article_id(self.pool.clone(), article.article_id)
                        .await
                        .map_err(AppError::DatabaseError)?;
                    article_dtos.push(ArticleDto::from_article_with_pictures(
                        article, pictures, tags,
                    ));
                }
                Ok(article_dtos)
            }
            Err(err) => {
                tracing::error!("Error fetching articles: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn get_articles_by_owner(&self, owner_id: Uuid) -> Result<Vec<ArticleDto>, AppError> {
        match self.repo.find_by_owner(self.pool.clone(), owner_id).await {
            Ok(articles) => {
                let mut article_dtos: Vec<ArticleDto> = Vec::with_capacity(articles.len());
                for article in articles {
                    let pictures = self
                        .file_repo
                        .find_by_article_id(self.pool.clone(), article.article_id)
                        .await
                        .map_err(AppError::DatabaseError)?;
                    let tags = self
                        .repo
                        .find_tags_by_article_id(self.pool.clone(), article.article_id)
                        .await
                        .map_err(AppError::DatabaseError)?;
                    article_dtos.push(ArticleDto::from_article_with_pictures(
                        article, pictures, tags,
                    ));
                }
                Ok(article_dtos)
            }
            Err(err) => {
                tracing::error!("Error fetching owner articles: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn get_categories(&self) -> Result<Vec<ArticleRelationDto>, AppError> {
        self.repo
            .find_categories(self.pool.clone())
            .await
            .map_err(AppError::DatabaseError)
    }

    async fn get_towns(&self) -> Result<Vec<ArticleRelationDto>, AppError> {
        self.repo
            .find_towns(self.pool.clone())
            .await
            .map_err(AppError::DatabaseError)
    }

    /// Creates a new article.
    async fn create_article(
        &self,
        create_article: CreateArticleDto,
    ) -> Result<ArticleDto, AppError> {
        let mut client = self.pool.get().await.unwrap();
        let mut tx = client.transaction().await.unwrap();
        let id = Uuid::new_v4();
        let tags =
            normalize_article_tags(&create_article.tags).map_err(AppError::ValidationError)?;
        let actor_id = create_article.created_by;
        match self.repo.create(&mut tx, id, create_article).await {
            Ok(()) => {
                if let Err(err) = self.repo.replace_tags(&mut tx, id, &tags, actor_id).await {
                    tracing::error!("Error assigning article tags: {err}");
                    let _ = tx.rollback().await;
                    return Err(AppError::DatabaseError(err));
                }
                tx.commit().await.map_err(|err| {
                    tracing::error!("Error committing article creation: {err}");
                    AppError::InternalError
                })?;
            }
            Err(err) => {
                let source = err
                    .source()
                    .map(|source| source.to_string())
                    .unwrap_or_else(|| "unknown source".to_string());
                tracing::error!(
                    "Error creating article: {err}; source: {source}; debug: {:?}",
                    err
                );
                let _ = tx.rollback().await;
                return Err(AppError::DatabaseError(err));
            }
        }

        match self.repo.find_by_id(self.pool.clone(), id).await {
            Ok(Some(article)) => {
                let pictures = self
                    .file_repo
                    .find_by_article_id(self.pool.clone(), id)
                    .await
                    .map_err(AppError::DatabaseError)?;
                let tags = self
                    .repo
                    .find_tags_by_article_id(self.pool.clone(), id)
                    .await
                    .map_err(AppError::DatabaseError)?;
                Ok(ArticleDto::from_article_with_pictures(
                    article, pictures, tags,
                ))
            }
            Ok(None) => Err(AppError::NotFound("Article not found".into())),
            Err(err) => {
                tracing::error!("Error retrieving article: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    /// Updates an existing article.
    async fn update_article(
        &self,
        id: Uuid,
        payload: UpdateArticleDtoWithIdDto,
    ) -> Result<ArticleDto, AppError> {
        let mut client = self.pool.get().await.unwrap();
        let mut tx = client.transaction().await.unwrap();
        let tags = normalize_article_tags(&payload.tags).map_err(AppError::ValidationError)?;
        let actor_id = payload.modified_by;

        match self.repo.update(&mut tx, id, payload).await {
            Ok(Some(article)) => {
                if let Err(err) = self.repo.replace_tags(&mut tx, id, &tags, actor_id).await {
                    tracing::error!("Error assigning article tags: {err}");
                    let _ = tx.rollback().await;
                    return Err(AppError::DatabaseError(err));
                }
                tx.commit().await.map_err(|err| {
                    tracing::error!("Error committing article update: {err}");
                    AppError::InternalError
                })?;
                let pictures = self
                    .file_repo
                    .find_by_article_id(self.pool.clone(), id)
                    .await
                    .map_err(AppError::DatabaseError)?;
                let tags = self
                    .repo
                    .find_tags_by_article_id(self.pool.clone(), id)
                    .await
                    .map_err(AppError::DatabaseError)?;
                Ok(ArticleDto::from_article_with_pictures(
                    article, pictures, tags,
                ))
            }
            Ok(None) => {
                let _ = tx.rollback().await;
                Err(AppError::NotFound("Article not found".into()))
            }
            Err(err) => {
                let source = err
                    .source()
                    .map(|source| source.to_string())
                    .unwrap_or_else(|| "unknown source".to_string());
                tracing::error!(
                    "Error updating article: {err}; source: {source}; debug: {:?}",
                    err
                );
                let _ = tx.rollback().await;
                Err(AppError::DatabaseError(err))
            }
        }
    }

    /// Deletes a article by their ID.
    async fn delete_article(&self, id: Uuid) -> Result<String, AppError> {
        let mut client = self.pool.get().await.unwrap();
        let mut tx = client.transaction().await.unwrap();

        match self.repo.delete(&mut tx, id).await {
            Ok(true) => {
                tx.commit().await.map_err(|err| {
                    tracing::error!("Error committing article deletion: {err}");
                    AppError::InternalError
                })?;
                Ok("Article deleted".into())
            }
            Ok(false) => {
                let _ = tx.rollback().await;
                Err(AppError::NotFound("Article not found".into()))
            }
            Err(err) => {
                let source = err
                    .source()
                    .map(|source| source.to_string())
                    .unwrap_or_else(|| "unknown source".to_string());
                tracing::error!(
                    "Error deleting article: {err}; source: {source}; debug: {:?}",
                    err
                );
                let _ = tx.rollback().await;
                Err(AppError::DatabaseError(err))
            }
        }
    }
}
