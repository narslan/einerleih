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
    pub mod calendar_dto;
}

mod infra {
    mod impl_repository;
    pub mod impl_service;
}

pub use api::routes::{CalendarApiDoc, protected_calendar_routes, user_calendar_routes};
pub use domain::model::CalendarEntrySource;
pub use domain::service::CalendarServiceTrait;
pub use infra::impl_service::CalendarService;
