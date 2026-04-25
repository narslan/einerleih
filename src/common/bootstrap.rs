use std::sync::Arc;

use deadpool_postgres::Pool;

use crate::common::config::Config;
use crate::domains::article::{ArticleServiceTrait, ArticleWorkflowServiceTrait};
use crate::domains::auth::{AuthService, AuthServiceTrait};
use crate::domains::booking::{BookingService, BookingServiceTrait};
use crate::domains::calendar::{CalendarService, CalendarServiceTrait};
use crate::domains::file::FileServiceTrait;
use crate::domains::mailbox::{MailboxService, MailboxServiceTrait};
use crate::domains::notification::{NotificationService, NotificationServiceTrait};
use crate::domains::user::UserServiceTrait;
use crate::{
    common::app_state::AppState,
    domains::article::{ArticleService, ArticleWorkflowService},
    domains::file::FileService,
    domains::user::UserService,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Constructs and wires all application services and returns a configured AppState.
pub fn build_app_state(pool: Pool, config: Config) -> AppState {
    let auth_service: Arc<dyn AuthServiceTrait> = AuthService::create_service(pool.clone());
    let user_service: Arc<dyn UserServiceTrait> = UserService::create_service(pool.clone());
    let file_service: Arc<dyn FileServiceTrait> =
        FileService::create_service(config.clone(), pool.clone());
    let article_service: Arc<dyn ArticleServiceTrait> =
        ArticleService::create_service(pool.clone());
    let article_workflow_service: Arc<dyn ArticleWorkflowServiceTrait> =
        ArticleWorkflowService::create_service(pool.clone(), file_service.clone());
    let calendar_service: Arc<dyn CalendarServiceTrait> =
        CalendarService::create_service(pool.clone());
    let mailbox_service: Arc<dyn MailboxServiceTrait> =
        MailboxService::create_service(pool.clone());
    let notification_service: Arc<dyn NotificationServiceTrait> =
        NotificationService::create_service(config.clone(), pool.clone());
    let booking_service: Arc<dyn BookingServiceTrait> =
        BookingService::create_service(pool.clone(), notification_service.clone());
    AppState::new(
        pool,
        config,
        auth_service,
        user_service,
        article_service,
        article_workflow_service,
        file_service,
        calendar_service,
        booking_service,
        mailbox_service,
        notification_service,
    )
}

/// Setup tracing for the application.
/// This function initializes the tracing subscriber with a default filter and formatting.
pub fn setup_tracing() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,axum::rejection=warn".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .without_time()
                .with_file(false)
                .with_line_number(false)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_target(false),
        )
        .init();
}

/// Shutdown signal handler
/// This function listens for a shutdown signal (CTRL+C) and logs a message when received.
pub async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}
