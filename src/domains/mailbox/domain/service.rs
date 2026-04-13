use std::sync::Arc;

use async_trait::async_trait;
use deadpool_postgres::Pool;
use uuid::Uuid;

use crate::{
    common::error::AppError,
    domains::mailbox::dto::mailbox_dto::{
        CreateBookingRequestMailboxEntriesDto, MailboxEntryDto, MailboxEntryFilterDto,
    },
};

#[async_trait]
pub trait MailboxServiceTrait: Send + Sync {
    fn create_service(pool: Pool) -> Arc<dyn MailboxServiceTrait>
    where
        Self: Sized;

    async fn get_entries(
        &self,
        owner_id: Uuid,
        filter: MailboxEntryFilterDto,
    ) -> Result<Vec<MailboxEntryDto>, AppError>;

    async fn get_entry(
        &self,
        owner_id: Uuid,
        mailbox_entry_id: Uuid,
    ) -> Result<MailboxEntryDto, AppError>;

    async fn mark_as_read(
        &self,
        owner_id: Uuid,
        mailbox_entry_id: Uuid,
    ) -> Result<MailboxEntryDto, AppError>;

    async fn create_booking_request_entries(
        &self,
        payload: CreateBookingRequestMailboxEntriesDto,
    ) -> Result<(), AppError>;
}
