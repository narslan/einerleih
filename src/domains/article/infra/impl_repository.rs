use async_trait::async_trait;

use deadpool_postgres::{Pool, PoolError, Transaction};
use tokio_postgres::Row;
use uuid::Uuid;

use crate::domains::article::domain::model::{Article, ArticleStatus};
use crate::domains::article::domain::repository::ArticleRepository;
use crate::domains::article::dto::article_dto::{
    ArticleRelationDto, ArticleTagDto, CreateArticleDto, NormalizedArticleTag,
    UpdateArticleDtoWithIdDto,
};

pub struct ArticleRepo;

const FIND_ARTICLE_BASE_QUERY: &str = r#"
    select
        article.article_id,
        article.name,
        article.category,
        categories.name as category_name,
        article.description,
        article.town,
        towns.name as town_name,
        article.status,
        article.created_by,
        article.created_at,
        article.modified_by,
        article.modified_at
    from article
    join categories on categories.category_id = article.category
    join towns on towns.town_id = article.town
    "#;

const FIND_ARTICLE_BY_ID_QUERY: &str = r#"
    select
        article.article_id,
        article.name,
        article.category,
        categories.name as category_name,
        article.description,
        article.town,
        towns.name as town_name,
        article.status,
        article.created_by,
        article.created_at,
        article.modified_by,
        article.modified_at
    from article
    join categories on categories.category_id = article.category
    join towns on towns.town_id = article.town
    where article.article_id = $1
    "#;

const FIND_ARTICLES_BY_OWNER_QUERY: &str = r#"
    select
        article.article_id,
        article.name,
        article.category,
        categories.name as category_name,
        article.description,
        article.town,
        towns.name as town_name,
        article.status,
        article.created_by,
        article.created_at,
        article.modified_by,
        article.modified_at
    from article
    join categories on categories.category_id = article.category
    join towns on towns.town_id = article.town
    where article.created_by = $1
    order by article.created_at desc, article.article_id desc
    "#;

const CREATE_ARTICLE_QUERY: &str = r#"
            INSERT INTO article 
            (article_id, name, category, description, town, status, created_by, modified_by)
            VALUES 
            ($1, $2, $3, $4, $5, $6, $7, $8)
            "#;

const UPDATE_ARTICLE_QUERY: &str = r#"
    WITH updated_article AS (
        UPDATE article
        SET
            name = $2,
            category = $3,
            description = $4,
            town = $5,
            status = $6,
            modified_by = $7,
            modified_at = NOW()
        WHERE article_id = $1
        RETURNING
            article_id,
            name,
            category,
            description,
            town,
            status,
            created_by,
            created_at,
            modified_by,
            modified_at
    )
    SELECT
        updated_article.article_id,
        updated_article.name,
        updated_article.category,
        categories.name as category_name,
        updated_article.description,
        updated_article.town,
        towns.name as town_name,
        updated_article.status,
        updated_article.created_by,
        updated_article.created_at,
        updated_article.modified_by,
        updated_article.modified_at
    FROM updated_article
    JOIN categories ON categories.category_id = updated_article.category
    JOIN towns ON towns.town_id = updated_article.town
    "#;

const DELETE_ARTICLE_QUERY: &str = r#"
    DELETE FROM article
    WHERE article_id = $1
    "#;

const FIND_CATEGORIES_QUERY: &str = r#"
    SELECT category_id, name
    FROM categories
    ORDER BY name ASC, category_id ASC
    "#;

const FIND_TOWNS_QUERY: &str = r#"
    SELECT town_id, name
    FROM towns
    ORDER BY name ASC, town_id ASC
    "#;

const FIND_TAGS_BY_ARTICLE_ID_QUERY: &str = r#"
    SELECT tags.tag_id, tags.slug, tags.name
    FROM tags
    JOIN article_tags ON article_tags.tag_id = tags.tag_id
    WHERE article_tags.article_id = $1
    ORDER BY tags.name ASC, tags.tag_id ASC
    "#;

const DELETE_ARTICLE_TAGS_QUERY: &str = r#"
    DELETE FROM article_tags
    WHERE article_id = $1
    "#;

const UPSERT_TAG_QUERY: &str = r#"
    INSERT INTO tags (tag_id, slug, name, created_by)
    VALUES ($1, $2, $3, $4)
    ON CONFLICT (slug) DO UPDATE
    SET name = tags.name
    RETURNING tag_id
    "#;

const INSERT_ARTICLE_TAG_QUERY: &str = r#"
    INSERT INTO article_tags (article_id, tag_id)
    VALUES ($1, $2)
    ON CONFLICT (article_id, tag_id) DO NOTHING
    "#;

fn map_article_row(row: &Row) -> Article {
    let status: String = row.get(7);

    Article {
        article_id: row.get(0),
        name: row.get(1),
        category_id: row.get(2),
        category_name: row.get(3),
        description: row.get(4),
        town_id: row.get(5),
        town_name: row.get(6),
        status: ArticleStatus::from_db(&status)
            .unwrap_or_else(|| panic!("invalid article status in database: {status}")),
        created_by: row.get(8),
        created_at: row.get(9),
        modified_by: row.get(10),
        modified_at: row.get(11),
    }
}

#[async_trait]
impl ArticleRepository for ArticleRepo {
    async fn find_all(&self, pool: Pool) -> Result<Vec<Article>, PoolError> {
        let client = pool.get().await?;
        let stmt = client.prepare_cached(FIND_ARTICLE_BASE_QUERY).await?;
        let rows = client.query(&stmt, &[]).await?;
        let articles = rows
            .into_iter()
            .map(|row| map_article_row(&row))
            .collect::<Vec<_>>();
        Ok(articles)
    }

    async fn find_by_id(&self, pool: Pool, id: Uuid) -> Result<Option<Article>, PoolError> {
        let client = pool.get().await?;
        let stmt = client.prepare_cached(FIND_ARTICLE_BY_ID_QUERY).await?;
        let row = client.query_opt(&stmt, &[&id]).await?;
        Ok(row.map(|row| map_article_row(&row)))
    }

    async fn find_by_owner(&self, pool: Pool, owner_id: Uuid) -> Result<Vec<Article>, PoolError> {
        let client = pool.get().await?;
        let stmt = client.prepare_cached(FIND_ARTICLES_BY_OWNER_QUERY).await?;
        let rows = client.query(&stmt, &[&owner_id]).await?;
        Ok(rows.into_iter().map(|row| map_article_row(&row)).collect())
    }

    async fn find_categories(&self, pool: Pool) -> Result<Vec<ArticleRelationDto>, PoolError> {
        let client = pool.get().await?;
        let stmt = client.prepare_cached(FIND_CATEGORIES_QUERY).await?;
        let rows = client.query(&stmt, &[]).await?;
        Ok(rows
            .into_iter()
            .map(|row| ArticleRelationDto {
                id: row.get(0),
                name: row.get(1),
            })
            .collect())
    }

    async fn find_towns(&self, pool: Pool) -> Result<Vec<ArticleRelationDto>, PoolError> {
        let client = pool.get().await?;
        let stmt = client.prepare_cached(FIND_TOWNS_QUERY).await?;
        let rows = client.query(&stmt, &[]).await?;
        Ok(rows
            .into_iter()
            .map(|row| ArticleRelationDto {
                id: row.get(0),
                name: row.get(1),
            })
            .collect())
    }

    async fn find_tags_by_article_id(
        &self,
        pool: Pool,
        article_id: Uuid,
    ) -> Result<Vec<ArticleTagDto>, PoolError> {
        let client = pool.get().await?;
        let stmt = client.prepare_cached(FIND_TAGS_BY_ARTICLE_ID_QUERY).await?;
        let rows = client.query(&stmt, &[&article_id]).await?;
        Ok(rows
            .into_iter()
            .map(|row| ArticleTagDto {
                id: row.get(0),
                slug: row.get(1),
                name: row.get(2),
            })
            .collect())
    }

    async fn replace_tags(
        &self,
        tx: &mut Transaction<'_>,
        article_id: Uuid,
        tags: &[NormalizedArticleTag],
        actor_id: Uuid,
    ) -> Result<(), PoolError> {
        let delete_stmt = tx.prepare_cached(DELETE_ARTICLE_TAGS_QUERY).await?;
        tx.execute(&delete_stmt, &[&article_id]).await?;

        let upsert_tag_stmt = tx.prepare_cached(UPSERT_TAG_QUERY).await?;
        let insert_article_tag_stmt = tx.prepare_cached(INSERT_ARTICLE_TAG_QUERY).await?;

        for tag in tags {
            let tag_id = Uuid::new_v4();
            let row = tx
                .query_one(
                    &upsert_tag_stmt,
                    &[&tag_id, &tag.slug, &tag.name, &actor_id],
                )
                .await?;
            let stored_tag_id: Uuid = row.get(0);
            tx.execute(&insert_article_tag_stmt, &[&article_id, &stored_tag_id])
                .await?;
        }

        Ok(())
    }

    async fn create(
        &self,
        tx: &mut Transaction<'_>,
        id: Uuid,
        article: CreateArticleDto,
    ) -> Result<(), PoolError> {
        let stmt = tx.prepare_cached(CREATE_ARTICLE_QUERY).await?;
        tx.execute(
            &stmt,
            &[
                &id,
                &article.name,
                &article.category,
                &article.description,
                &article.town,
                &article.status.as_str(),
                &article.created_by,
                &article.modified_by,
            ],
        )
        .await?;
        Ok(())
    }

    async fn update(
        &self,
        tx: &mut Transaction<'_>,
        id: uuid::Uuid,
        article: UpdateArticleDtoWithIdDto,
    ) -> Result<Option<Article>, PoolError> {
        let stmt = tx.prepare_cached(UPDATE_ARTICLE_QUERY).await?;
        let row = tx
            .query_opt(
                &stmt,
                &[
                    &id,
                    &article.name,
                    &article.category,
                    &article.description,
                    &article.town,
                    &article.status.as_str(),
                    &article.modified_by,
                ],
            )
            .await?;

        Ok(row.map(|row| map_article_row(&row)))
    }

    async fn delete(&self, tx: &mut Transaction<'_>, id: Uuid) -> Result<bool, PoolError> {
        let stmt = tx.prepare_cached(DELETE_ARTICLE_QUERY).await?;
        let affected_rows = tx.execute(&stmt, &[&id]).await?;
        Ok(affected_rows > 0)
    }
}
