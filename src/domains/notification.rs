mod domain {
    pub mod email_sender;
    pub mod model;
    pub mod repository;
    pub mod service;
}

pub mod dto {
    pub mod notification_dto;
}

mod infra {
    pub mod file_email_sender;
    pub mod impl_repository;
    pub mod impl_service;
}

pub use domain::email_sender::EmailSender;
pub use domain::model::{NotificationKind, NotificationOutboxEntry, NotificationStatus};
pub use domain::repository::NotificationOutboxRepository;
pub use domain::service::NotificationServiceTrait;
pub use infra::file_email_sender::FileEmailSender;
pub use infra::impl_repository::NotificationOutboxRepo;
pub use infra::impl_service::NotificationService;
