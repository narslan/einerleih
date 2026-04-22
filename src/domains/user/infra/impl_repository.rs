use crate::domains::user::{
    domain::{model::User, repository::UserRepository},
    dto::user_dto::{CreateUserDto, SearchUserDto, UpdateUserDto},
};
use async_trait::async_trait;

use deadpool_postgres::{Pool, PoolError};
use tokio_postgres::{Row, types::ToSql};
use uuid::Uuid;

pub struct UserRepo;

const FIND_USER_BASE_QUERY: &str = r#"
    SELECT
        u.id,
        u.username,
        u.email,
        u.created_by,
        u.created_at,
        u.modified_by,
        u.modified_at
    FROM users u
    WHERE 1=1
    "#;

const FIND_USER_BY_ID_QUERY: &str = r#"
    SELECT
        u.id,
        u.username,
        u.email,
        u.created_by,
        u.created_at,
        u.modified_by,
        u.modified_at
    FROM users u
    WHERE u.id = $1
    "#;

const FIND_USER_BY_EMAIL_QUERY: &str = r#"
    SELECT
        u.id,
        u.username,
        u.email,
        u.created_by,
        u.created_at,
        u.modified_by,
        u.modified_at
    FROM users u
    WHERE LOWER(u.email) = LOWER($1)
    LIMIT 1
    "#;

const CREATE_USER_QUERY: &str = r#"
    INSERT INTO users (id, username, email, created_by, modified_by)
    VALUES ($1, $2, $3, $4, $5)
    "#;

const UPDATE_USER_QUERY: &str = r#"
    UPDATE users
    SET
        username = $2,
        email = $3,
        modified_by = $4,
        modified_at = NOW()
    WHERE id = $1
    RETURNING
        id,
        username,
        email,
        created_by,
        created_at,
        modified_by,
        modified_at
    "#;

const DELETE_USER_QUERY: &str = r#"
    DELETE FROM users
    WHERE id = $1
    "#;

fn map_user_row(row: &Row) -> User {
    User {
        id: row.get(0),
        username: row.get(1),
        email: row.get(2),
        created_by: row.get(3),
        created_at: row.get(4),
        modified_by: row.get(5),
        modified_at: row.get(6),
    }
}

#[async_trait]
impl UserRepository for UserRepo {
    async fn find_all(&self, pool: Pool) -> Result<Vec<User>, PoolError> {
        let client = pool.get().await?;
        let stmt = client.prepare_cached(FIND_USER_BASE_QUERY).await?;
        let rows = client.query(&stmt, &[]).await?;
        let users = rows
            .into_iter()
            .map(|row| map_user_row(&row))
            .collect::<Vec<_>>();
        Ok(users)
    }

    async fn find_list(
        &self,
        pool: Pool,
        search_user_dto: SearchUserDto,
    ) -> Result<Vec<User>, PoolError> {
        let client = pool.get().await?;
        let mut query = String::from(FIND_USER_BASE_QUERY);
        let mut param_storage: Vec<Box<dyn ToSql + Sync + Send>> = Vec::new();

        if let Some(id) = search_user_dto.id {
            param_storage.push(Box::new(id));
            query.push_str(&format!(" AND u.id = ${}", param_storage.len()));
        }

        if let Some(username) = search_user_dto
            .username
            .filter(|value| !value.trim().is_empty())
        {
            param_storage.push(Box::new(format!("%{}%", username.trim())));
            query.push_str(&format!(" AND u.username ILIKE ${}", param_storage.len()));
        }

        if let Some(email) = search_user_dto
            .email
            .filter(|value| !value.trim().is_empty())
        {
            param_storage.push(Box::new(email));
            query.push_str(&format!(" AND u.email = ${}", param_storage.len()));
        }

        let stmt = client.prepare(&query).await?;
        let params: Vec<&(dyn ToSql + Sync)> = param_storage
            .iter()
            .map(|param| param.as_ref() as &(dyn ToSql + Sync))
            .collect();
        let rows = client.query(&stmt, &params).await?;
        Ok(rows.into_iter().map(|row| map_user_row(&row)).collect())
    }

    async fn find_by_id(&self, pool: Pool, id: Uuid) -> Result<Option<User>, PoolError> {
        let client = pool.get().await?;
        let stmt = client.prepare_cached(FIND_USER_BY_ID_QUERY).await?;
        let row = client.query_opt(&stmt, &[&id]).await?;
        Ok(row.map(|row| map_user_row(&row)))
    }

    async fn find_by_email(&self, pool: Pool, email: &str) -> Result<Option<User>, PoolError> {
        let client = pool.get().await?;
        let stmt = client.prepare_cached(FIND_USER_BY_EMAIL_QUERY).await?;
        let row = client.query_opt(&stmt, &[&email]).await?;
        Ok(row.map(|row| map_user_row(&row)))
    }

    async fn create(&self, pool: Pool, user: CreateUserDto) -> Result<Uuid, PoolError> {
        let id = Uuid::new_v4();
        let client = pool.get().await?;
        let stmt = client.prepare_cached(CREATE_USER_QUERY).await?;

        client
            .execute(
                &stmt,
                &[
                    &id,
                    &user.username,
                    &user.email,
                    &user.modified_by,
                    &user.modified_by,
                ],
            )
            .await?;

        Ok(id)
    }

    async fn update(
        &self,
        pool: Pool,
        id: Uuid,
        user: UpdateUserDto,
    ) -> Result<Option<User>, PoolError> {
        let client = pool.get().await?;
        let stmt = client.prepare_cached(UPDATE_USER_QUERY).await?;
        let row = client
            .query_opt(
                &stmt,
                &[&id, &user.username, &user.email, &user.modified_by],
            )
            .await?;

        Ok(row.map(|row| map_user_row(&row)))
    }

    async fn delete(&self, pool: Pool, id: Uuid) -> Result<bool, PoolError> {
        let client = pool.get().await?;
        let stmt = client.prepare_cached(DELETE_USER_QUERY).await?;
        let affected_rows = client.execute(&stmt, &[&id]).await?;
        Ok(affected_rows > 0)
    }
}
