use einerleih::{
    app::create_router,
    common::{
        bootstrap::{build_app_state, setup_tracing, shutdown_signal},
        config::{self as app_config, setup_database},
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    setup_tracing();
    let config = app_config::Config::from_env()?;

    let pool = setup_database(&config).await?;
    let state = build_app_state(pool, config.clone());
    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind(&config.listen).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
