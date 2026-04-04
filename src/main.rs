use einerleih::{
    app::create_router,
    common::{
        bootstrap::{build_app_state, setup_tracing, shutdown_signal},
        config::{self as app_config, setup_database},
        db_migrations,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    setup_tracing();
    let config = app_config::Config::from_env()?;
    let pool = setup_database(&config).await?;
    let args: Vec<String> = std::env::args().collect();

    if let Some(command) = args.get(1) {
        match command.as_str() {
            "migrate" => {
                let subcommand = args.get(2).map(String::as_str).unwrap_or("up");
                return run_migration_command(&pool, subcommand).await;
            }
            "seed" => {
                db_migrations::run_seed(&pool).await?;
                println!("Seed data applied.");
                return Ok(());
            }
            _ => {}
        }
    }

    db_migrations::apply_schema(&pool).await?;
    let state = build_app_state(pool, config.clone());
    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind(&config.listen).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn run_migration_command(
    pool: &deadpool_postgres::Pool,
    subcommand: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match subcommand {
        "up" => {
            db_migrations::apply_schema(pool).await?;
            println!("Applied pending migrations.");
        }
        "down" => {
            if db_migrations::revert_last_migration(pool).await? {
                println!("Reverted last migration.");
            } else {
                println!("No applied migrations to revert.");
            }
        }
        "reset" => {
            db_migrations::reset_schema(pool).await?;
            println!("Schema reset complete.");
        }
        "status" => {
            for (version, applied) in db_migrations::migration_status(pool).await? {
                let status = if applied { "applied" } else { "pending" };
                println!("{version}: {status}");
            }
        }
        other => {
            return Err(format!(
                "unknown migration subcommand '{other}', expected one of: up, down, reset, status"
            )
            .into());
        }
    }

    Ok(())
}
