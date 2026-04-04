use std::{fs, path::PathBuf};

use deadpool_postgres::Pool;

type MigrationResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const DROP_SCHEMA_SQL: &str = "db-seeds/00-drop-tables.sql";
const CREATE_SCHEMA_SQL: &str = "db-seeds/01-tables.sql";

fn project_file(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path)
}

fn read_sql_file(relative_path: &str) -> MigrationResult<String> {
    Ok(fs::read_to_string(project_file(relative_path))?)
}

pub async fn run_sql_file(pool: &Pool, relative_path: &str) -> MigrationResult<()> {
    let sql = read_sql_file(relative_path)?;
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;
    transaction.batch_execute(&sql).await?;
    transaction.commit().await?;
    Ok(())
}

pub async fn drop_schema(pool: &Pool) -> MigrationResult<()> {
    run_sql_file(pool, DROP_SCHEMA_SQL).await
}

pub async fn apply_schema(pool: &Pool) -> MigrationResult<()> {
    run_sql_file(pool, CREATE_SCHEMA_SQL).await
}

pub async fn reset_schema(pool: &Pool) -> MigrationResult<()> {
    drop_schema(pool).await?;
    apply_schema(pool).await?;
    Ok(())
}
