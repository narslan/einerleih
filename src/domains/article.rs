mod api {
    mod handlers;
    pub mod routes;
}

mod domain {
    pub mod model;
    pub mod repository;
    pub mod service;
    pub mod workflow;
}

pub mod dto {
    pub mod article_dto;
}

mod infra {
    mod impl_repository;
    pub mod impl_service;
    pub mod impl_workflow_service;
}

// Re-export commonly used items for convenience
pub use api::routes::{ArticleApiDoc, protected_article_routes, public_article_routes};
pub use domain::model::ArticleStatus;
pub use domain::service::ArticleServiceTrait;
pub use domain::workflow::ArticleWorkflowServiceTrait;
pub use infra::impl_service::ArticleService;
pub use infra::impl_workflow_service::ArticleWorkflowService;
