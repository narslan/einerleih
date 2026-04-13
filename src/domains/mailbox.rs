mod api {
    mod handlers;
    pub mod routes;
}

mod domain {
    pub mod model;
    pub mod repository;
    pub mod service;
}

pub mod dto {
    pub mod mailbox_dto;
}

mod infra {
    pub mod impl_repository;
    pub mod impl_service;
}

pub use api::routes::{MailboxApiDoc, user_mailbox_routes};
pub use domain::model::MailboxDirection;
pub use domain::repository::MailboxRepository;
pub use domain::service::MailboxServiceTrait;
pub use infra::impl_repository::MailboxRepo;
pub use infra::impl_service::MailboxService;
