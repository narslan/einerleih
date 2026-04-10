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
    pub mod booking_dto;
}

mod infra {
    mod impl_repository;
    pub mod impl_service;
}

pub use api::routes::{BookingApiDoc, protected_booking_routes};
pub use domain::model::BookingStatus;
pub use domain::service::BookingServiceTrait;
pub use infra::impl_service::BookingService;
