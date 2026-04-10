use std::sync::Arc;

use crate::domains::{
    article::{ArticleServiceTrait, ArticleWorkflowServiceTrait},
    auth::AuthServiceTrait,
    booking::BookingServiceTrait,
    calendar::CalendarServiceTrait,
    file::FileServiceTrait,
    user::UserServiceTrait,
};

use super::config::Config;

#[derive(Clone)]
pub struct AppState {
    /// Service handling authentication-related logic.
    pub auth_service: Arc<dyn AuthServiceTrait>,
    /// Service handling user-related logic.
    pub user_service: Arc<dyn UserServiceTrait>,

    /// Service handling article-related logic.
    pub article_service: Arc<dyn ArticleServiceTrait>,

    /// Service orchestrating article creation together with related uploads.
    pub article_workflow_service: Arc<dyn ArticleWorkflowServiceTrait>,

    /// Service handling file-related logic.
    pub file_service: Arc<dyn FileServiceTrait>,

    /// Service handling article calendar entries.
    pub calendar_service: Arc<dyn CalendarServiceTrait>,

    /// Service handling booking requests and status transitions.
    pub booking_service: Arc<dyn BookingServiceTrait>,

    /// Global application configuration.
    pub config: Config,
}

impl AppState {
    pub fn new(
        config: Config,
        auth_service: Arc<dyn AuthServiceTrait>,
        user_service: Arc<dyn UserServiceTrait>,
        article_service: Arc<dyn ArticleServiceTrait>,
        article_workflow_service: Arc<dyn ArticleWorkflowServiceTrait>,
        file_service: Arc<dyn FileServiceTrait>,
        calendar_service: Arc<dyn CalendarServiceTrait>,
        booking_service: Arc<dyn BookingServiceTrait>,
    ) -> Self {
        Self {
            config,
            auth_service,
            user_service,
            article_service,
            article_workflow_service,
            file_service,
            calendar_service,
            booking_service,
        }
    }
}
