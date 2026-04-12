use std::collections::HashSet;

use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_login::{AuthUser, AuthnBackend, AuthzBackend, UserId};
use deadpool_postgres::{Pool, PoolError};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;

use crate::common::{auth_context::AuthenticatedUser, config::Config, error::AppError, hash_utils};
use crate::domains::auth::dto::auth_dto::AuthPayload;

pub type AuthSession = axum_login::AuthSession<AuthBackend>;

pub const ROLE_ADMIN: &str = "admin";
pub const ROLE_USER: &str = "user";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUser {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub roles: HashSet<String>,
    password_hash: String,
}

impl AuthUser for SessionUser {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash.as_bytes()
    }
}

#[derive(Clone)]
pub struct AuthBackend {
    pool: Pool,
}

impl AuthBackend {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<SessionUser>, PoolError> {
        let client = self.pool.get().await?;
        let stmt = client.prepare_cached(FIND_SESSION_USER_BY_USERNAME).await?;
        let row = client.query_opt(&stmt, &[&username]).await?;
        Ok(row.map(|row| map_session_user_row(&row)))
    }

    async fn find_by_id(&self, user_id: &Uuid) -> Result<Option<SessionUser>, PoolError> {
        let client = self.pool.get().await?;
        let stmt = client.prepare_cached(FIND_SESSION_USER_BY_ID).await?;
        let row = client.query_opt(&stmt, &[&user_id]).await?;
        Ok(row.map(|row| map_session_user_row(&row)))
    }
}

impl AuthnBackend for AuthBackend {
    type User = SessionUser;
    type Credentials = AuthPayload;
    type Error = AppError;

    async fn authenticate(
        &self,
        credentials: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let username = credentials.client_id.trim();
        if username.is_empty() || credentials.client_secret.is_empty() {
            return Err(AppError::MissingCredentials);
        }

        let Some(user) = self
            .find_by_username(username)
            .await
            .map_err(AppError::DatabaseError)?
        else {
            return Ok(None);
        };

        if !hash_utils::verify_password(&user.password_hash, &credentials.client_secret) {
            return Ok(None);
        }

        Ok(Some(user))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        self.find_by_id(user_id)
            .await
            .map_err(AppError::DatabaseError)
    }
}

impl AuthzBackend for AuthBackend {
    type Permission = String;

    async fn get_user_permissions(
        &self,
        user: &Self::User,
    ) -> Result<HashSet<Self::Permission>, Self::Error> {
        Ok(user.roles.clone())
    }
}

pub async fn require_session(
    auth_session: AuthSession,
    mut req: Request,
    next: Next,
) -> Result<Response, Response> {
    let Some(user) = auth_session.user else {
        return Err(AppError::Unauthorized.into_response());
    };

    req.extensions_mut().insert(authenticated_user(&user));

    Ok(next.run(req).await)
}

pub async fn require_admin(
    auth_session: AuthSession,
    mut req: Request,
    next: Next,
) -> Result<Response, Response> {
    let Some(user) = auth_session.user else {
        return Err(AppError::Unauthorized.into_response());
    };

    if !user.roles.contains(ROLE_ADMIN) {
        return Err(AppError::Forbidden.into_response());
    }

    req.extensions_mut().insert(authenticated_user(&user));

    Ok(next.run(req).await)
}

pub async fn assign_signup_role(pool: &Pool, user_id: Uuid) -> Result<(), AppError> {
    let role_name = if admin_exists(pool).await? {
        ROLE_USER
    } else {
        ROLE_ADMIN
    };
    assign_role(pool, user_id, role_name).await
}

pub async fn ensure_bootstrap_admin(pool: &Pool, config: &Config) -> Result<(), AppError> {
    let username = config
        .bootstrap_admin_username
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let password = config
        .bootstrap_admin_password
        .as_deref()
        .filter(|value| !value.is_empty());

    let Some(username) = username else {
        return Ok(());
    };
    let Some(password) = password else {
        return Err(AppError::ValidationError(
            "BOOTSTRAP_ADMIN_PASSWORD is required when BOOTSTRAP_ADMIN_USERNAME is set".into(),
        ));
    };

    let email = config
        .bootstrap_admin_email
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("{username}@einerleih.local"));
    let password_hash = hash_utils::hash_password(password).map_err(|err| {
        tracing::error!("Error hashing bootstrap admin password: {err}");
        AppError::InternalError
    })?;
    let user_id = upsert_bootstrap_admin_user(pool, username, &email, &password_hash).await?;
    assign_role(pool, user_id, ROLE_ADMIN).await?;

    tracing::info!(username = %username, "Bootstrap admin account ensured");

    Ok(())
}

async fn admin_exists(pool: &Pool) -> Result<bool, AppError> {
    let client = pool.get().await?;
    let stmt = client.prepare_cached(ADMIN_EXISTS).await?;
    let row = client.query_one(&stmt, &[&ROLE_ADMIN]).await?;
    Ok(row.get(0))
}

async fn assign_role(pool: &Pool, user_id: Uuid, role_name: &str) -> Result<(), AppError> {
    let client = pool.get().await?;
    let stmt = client.prepare_cached(ASSIGN_ROLE).await?;
    client.execute(&stmt, &[&user_id, &role_name]).await?;
    Ok(())
}

async fn upsert_bootstrap_admin_user(
    pool: &Pool,
    username: &str,
    email: &str,
    password_hash: &str,
) -> Result<Uuid, AppError> {
    let client = pool.get().await?;
    let stmt = client.prepare_cached(UPSERT_BOOTSTRAP_ADMIN_USER).await?;
    let user_id = Uuid::new_v4();
    let row = client
        .query_one(&stmt, &[&user_id, &username, &email, &password_hash])
        .await?;
    Ok(row.get(0))
}

fn authenticated_user(user: &SessionUser) -> AuthenticatedUser {
    AuthenticatedUser {
        id: user.id,
        roles: user.roles.clone(),
    }
}

const FIND_SESSION_USER_BY_USERNAME: &str = r#"
    SELECT
        u.id,
        u.username,
        u.email,
        ua.password_hash,
        COALESCE(
            ARRAY_AGG(r.name::TEXT ORDER BY r.name) FILTER (WHERE r.name IS NOT NULL),
            ARRAY[]::TEXT[]
        ) AS roles
    FROM user_auth ua
    JOIN users u ON ua.user_id = u.id
    LEFT JOIN user_roles ur ON ur.user_id = u.id
    LEFT JOIN roles r ON r.role_id = ur.role_id
    WHERE u.username = $1
    GROUP BY u.id, u.username, u.email, ua.password_hash
    "#;

const FIND_SESSION_USER_BY_ID: &str = r#"
    SELECT
        u.id,
        u.username,
        u.email,
        ua.password_hash,
        COALESCE(
            ARRAY_AGG(r.name::TEXT ORDER BY r.name) FILTER (WHERE r.name IS NOT NULL),
            ARRAY[]::TEXT[]
        ) AS roles
    FROM user_auth ua
    JOIN users u ON ua.user_id = u.id
    LEFT JOIN user_roles ur ON ur.user_id = u.id
    LEFT JOIN roles r ON r.role_id = ur.role_id
    WHERE u.id = $1
    GROUP BY u.id, u.username, u.email, ua.password_hash
    "#;

const ADMIN_EXISTS: &str = r#"
    SELECT EXISTS (
        SELECT 1
        FROM user_roles ur
        JOIN roles r ON r.role_id = ur.role_id
        WHERE r.name = $1
    )
    "#;

const ASSIGN_ROLE: &str = r#"
    INSERT INTO user_roles (user_id, role_id)
    SELECT $1, role_id
    FROM roles
    WHERE name = $2
    ON CONFLICT (user_id, role_id) DO NOTHING
    "#;

const UPSERT_BOOTSTRAP_ADMIN_USER: &str = r#"
    WITH upserted_user AS (
        INSERT INTO users (id, username, email, created_by, modified_by)
        VALUES ($1, $2, $3, NULL, NULL)
        ON CONFLICT (username) DO UPDATE
        SET
            email = EXCLUDED.email,
            modified_at = NOW()
        RETURNING id
    )
    INSERT INTO user_auth (user_id, password_hash)
    SELECT id, $4
    FROM upserted_user
    ON CONFLICT (user_id) DO UPDATE
    SET
        password_hash = EXCLUDED.password_hash,
        modified_at = NOW()
    RETURNING user_id
    "#;

fn map_session_user_row(row: &Row) -> SessionUser {
    let roles: Vec<String> = row.get(4);

    SessionUser {
        id: row.get(0),
        username: row.get(1),
        email: row.get(2),
        password_hash: row.get(3),
        roles: roles.into_iter().collect(),
    }
}
