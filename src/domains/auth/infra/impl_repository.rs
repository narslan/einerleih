use async_trait::async_trait;
use chrono::{DateTime, Utc};

use deadpool_postgres::{Pool, PoolError, Transaction};
use tokio_postgres::Row;
use uuid::Uuid;

use crate::domains::auth::domain::model::{EmailVerificationToken, UserAuth};
use crate::domains::auth::domain::repository::UserAuthRepository;
pub struct UserAuthRepo;

const FIND_BY_USER_NAME_QUERY: &str = r#"
            SELECT ua.user_id, ua.password_hash, (u.email_verified_at IS NOT NULL) AS email_verified
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

const CREATE_EMAIL_VERIFICATION_TOKEN_QUERY: &str = r#"
            INSERT INTO email_verification_token
            (verification_token_id, user_id, token, expires_at)
            VALUES
            ($1, $2, $3, $4)
            "#;

const FIND_ACTIVE_EMAIL_VERIFICATION_TOKEN_QUERY: &str = r#"
            SELECT
                verification_token_id,
                user_id,
                token,
                expires_at
            FROM email_verification_token
            WHERE token = $1
                AND consumed_at IS NULL
                AND expires_at > $2
            ORDER BY created_at DESC, verification_token_id DESC
            LIMIT 1
            "#;

const CONSUME_EMAIL_VERIFICATION_TOKEN_QUERY: &str = r#"
            UPDATE email_verification_token
            SET consumed_at = COALESCE(consumed_at, NOW())
            WHERE verification_token_id = $1
                AND consumed_at IS NULL
            "#;

const MARK_EMAIL_VERIFIED_QUERY: &str = r#"
            UPDATE users
            SET
                email_verified_at = COALESCE(email_verified_at, NOW()),
                modified_at = NOW()
            WHERE id = $1
            "#;

fn map_user_auth_row(row: &Row) -> UserAuth {
    UserAuth {
        user_id: row.get(0),
        password_hash: row.get(1),
        email_verified: row.get(2),
    }
}

fn map_email_verification_token_row(row: &Row) -> EmailVerificationToken {
    EmailVerificationToken {
        verification_token_id: row.get(0),
        user_id: row.get(1),
        token: row.get(2),
        expires_at: row.get(3),
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

    async fn create_email_verification_token(
        &self,
        tx: &mut Transaction<'_>,
        token: EmailVerificationToken,
    ) -> Result<(), PoolError> {
        let stmt = tx
            .prepare_cached(CREATE_EMAIL_VERIFICATION_TOKEN_QUERY)
            .await?;
        tx.execute(
            &stmt,
            &[
                &token.verification_token_id,
                &token.user_id,
                &token.token,
                &token.expires_at,
            ],
        )
        .await?;
        Ok(())
    }

    async fn find_active_email_verification_token(
        &self,
        pool: Pool,
        token: &str,
        now: DateTime<Utc>,
    ) -> Result<Option<EmailVerificationToken>, PoolError> {
        let client = pool.get().await?;
        let stmt = client
            .prepare_cached(FIND_ACTIVE_EMAIL_VERIFICATION_TOKEN_QUERY)
            .await?;
        let row = client.query_opt(&stmt, &[&token, &now]).await?;
        Ok(row.map(|row| map_email_verification_token_row(&row)))
    }

    async fn consume_email_verification_token(
        &self,
        tx: &mut Transaction<'_>,
        verification_token_id: Uuid,
    ) -> Result<bool, PoolError> {
        let stmt = tx
            .prepare_cached(CONSUME_EMAIL_VERIFICATION_TOKEN_QUERY)
            .await?;
        let affected = tx.execute(&stmt, &[&verification_token_id]).await?;
        Ok(affected > 0)
    }

    async fn mark_email_verified(
        &self,
        tx: &mut Transaction<'_>,
        user_id: Uuid,
    ) -> Result<bool, PoolError> {
        let stmt = tx.prepare_cached(MARK_EMAIL_VERIFIED_QUERY).await?;
        let affected = tx.execute(&stmt, &[&user_id]).await?;
        Ok(affected > 0)
    }
}
