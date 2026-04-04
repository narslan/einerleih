use async_trait::async_trait;

use deadpool_postgres::{Pool, PoolError, Transaction};
use tokio_postgres::Row;

use crate::domains::auth::domain::model::UserAuth;
use crate::domains::auth::domain::repository::UserAuthRepository;
pub struct UserAuthRepo;

const FIND_BY_USER_NAME_QUERY: &str = r#"
            SELECT ua.user_id, ua.password_hash
              FROM user_auth ua
              JOIN users u ON ua.user_id = u.id
              WHERE u.username = $1
            "#;

const CREATE_USER_QUERY: &str = r#"
            INSERT INTO user_auth 
            (user_id, password_hash)
            VALUES 
            ($1, $2)
            "#;

fn map_user_auth_row(row: &Row) -> UserAuth {
    UserAuth {
        user_id: row.get(0),
        password_hash: row.get(1),
    }
}

#[async_trait]
impl UserAuthRepository for UserAuthRepo {
    async fn find_by_user_name(
        &self,
        pool: Pool,
        user_name: String,
    ) -> Result<Option<UserAuth>, PoolError> {
        let client = pool.get().await?;
        let stmt = client.prepare_cached(FIND_BY_USER_NAME_QUERY).await?;
        let row = client.query_opt(&stmt, &[&user_name]).await?;
        Ok(row.map(|row| map_user_auth_row(&row)))
    }

    async fn create(&self, tx: &mut Transaction<'_>, user_auth: UserAuth) -> Result<(), PoolError> {
        let stmt = tx.prepare_cached(CREATE_USER_QUERY).await?;
        tx.execute(&stmt, &[&user_auth.user_id, &user_auth.password_hash])
            .await?;
        Ok(())
    }
}
