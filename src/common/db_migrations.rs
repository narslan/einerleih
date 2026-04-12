use std::{
    fs,
    path::{Path, PathBuf},
};

use deadpool_postgres::Pool;

type MigrationResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const MIGRATIONS_DIR: &str = "db/migrations";
const DEV_SEED_SQL: &str = "db/seeds/01-dev.sql";

#[derive(Debug, Clone)]
struct Migration {
    version: String,
    up_path: PathBuf,
    down_path: PathBuf,
}

fn runtime_file(relative_path: &str) -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(relative_path)
}

fn read_sql_file(path: &Path) -> MigrationResult<String> {
    Ok(fs::read_to_string(path)?)
}

fn load_migrations() -> MigrationResult<Vec<Migration>> {
    let directory = runtime_file(MIGRATIONS_DIR);
    let mut migrations = Vec::new();

    for entry in fs::read_dir(&directory)? {
        let path = entry?.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if !file_name.ends_with(".up.sql") {
            continue;
        }

        let version = file_name.trim_end_matches(".up.sql").to_string();
        let down_path = directory.join(format!("{version}.down.sql"));

        if !down_path.exists() {
            return Err(format!("missing down migration for {version}").into());
        }

        migrations.push(Migration {
            version,
            up_path: path,
            down_path,
        });
    }

    migrations.sort_by(|left, right| left.version.cmp(&right.version));
    Ok(migrations)
}

async fn ensure_migrations_table(pool: &Pool) -> MigrationResult<()> {
    let client = pool.get().await?;
    client
        .batch_execute(
            "
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version VARCHAR(255) PRIMARY KEY,
                applied_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            ",
        )
        .await?;
    Ok(())
}

async fn applied_versions(pool: &Pool) -> MigrationResult<Vec<String>> {
    ensure_migrations_table(pool).await?;
    let client = pool.get().await?;
    let rows = client
        .query(
            "SELECT version FROM schema_migrations ORDER BY version ASC",
            &[],
        )
        .await?;

    Ok(rows.into_iter().map(|row| row.get(0)).collect())
}

async fn apply_migration(pool: &Pool, migration: &Migration) -> MigrationResult<()> {
    let sql = read_sql_file(&migration.up_path)?;
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;
    transaction.batch_execute(&sql).await?;
    transaction
        .execute(
            "INSERT INTO schema_migrations (version) VALUES ($1)",
            &[&migration.version],
        )
        .await?;
    transaction.commit().await?;
    Ok(())
}

async fn revert_migration(pool: &Pool, migration: &Migration) -> MigrationResult<()> {
    let sql = read_sql_file(&migration.down_path)?;
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;
    transaction.batch_execute(&sql).await?;
    transaction
        .execute(
            "DELETE FROM schema_migrations WHERE version = $1",
            &[&migration.version],
        )
        .await?;
    transaction.commit().await?;
    Ok(())
}

pub async fn run_seed(pool: &Pool) -> MigrationResult<()> {
    let sql = read_sql_file(&runtime_file(DEV_SEED_SQL))?;
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;
    transaction.batch_execute(&sql).await?;
    transaction.commit().await?;
    Ok(())
}

pub async fn apply_schema(pool: &Pool) -> MigrationResult<()> {
    ensure_migrations_table(pool).await?;
    let migrations = load_migrations()?;
    let applied = applied_versions(pool).await?;

    for migration in migrations {
        if applied.iter().any(|version| version == &migration.version) {
            continue;
        }

        apply_migration(pool, &migration).await?;
    }

    Ok(())
}

pub async fn drop_schema(pool: &Pool) -> MigrationResult<()> {
    ensure_migrations_table(pool).await?;
    let applied = applied_versions(pool).await?;
    let migrations = load_migrations()?;

    for version in applied.into_iter().rev() {
        let migration = migrations
            .iter()
            .find(|migration| migration.version == version)
            .ok_or_else(|| format!("missing migration metadata for {version}"))?;
        revert_migration(pool, migration).await?;
    }

    Ok(())
}

pub async fn reset_schema(pool: &Pool) -> MigrationResult<()> {
    drop_application_tables(pool).await?;
    apply_schema(pool).await?;
    Ok(())
}

async fn drop_application_tables(pool: &Pool) -> MigrationResult<()> {
    let client = pool.get().await?;
    client
        .batch_execute(
            "
            DROP TABLE IF EXISTS uploaded_files;
            DROP TABLE IF EXISTS booking;
            DROP TABLE IF EXISTS event_calendar;
            DROP TABLE IF EXISTS user_auth;
            DROP TABLE IF EXISTS article;
            DROP TABLE IF EXISTS categories;
            DROP TABLE IF EXISTS towns;
            DROP TABLE IF EXISTS users;
            DROP TABLE IF EXISTS schema_migrations;
            ",
        )
        .await?;
    Ok(())
}

pub async fn migration_status(pool: &Pool) -> MigrationResult<Vec<(String, bool)>> {
    let applied = applied_versions(pool).await?;
    let migrations = load_migrations()?;

    Ok(migrations
        .into_iter()
        .map(|migration| {
            let is_applied = applied.iter().any(|version| version == &migration.version);
            (migration.version, is_applied)
        })
        .collect())
}

pub async fn revert_last_migration(pool: &Pool) -> MigrationResult<bool> {
    ensure_migrations_table(pool).await?;
    let applied = applied_versions(pool).await?;
    let Some(version) = applied.into_iter().last() else {
        return Ok(false);
    };

    let migrations = load_migrations()?;
    let migration = migrations
        .iter()
        .find(|migration| migration.version == version)
        .ok_or_else(|| format!("missing migration metadata for {version}"))?;

    revert_migration(pool, migration).await?;
    Ok(true)
}
