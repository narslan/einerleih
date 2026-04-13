use async_trait::async_trait;
use deadpool_postgres::{Pool, PoolError, Transaction};
use uuid::Uuid;

use crate::domains::mailbox::{
    domain::model::MailboxEntry,
    dto::mailbox_dto::{CreateBookingRequestMailboxEntriesDto, MailboxEntryFilterDto},
};

#[async_trait]
pub trait MailboxRepository: Send + Sync {
    async fn find_by_owner_id(
        &self,
        pool: Pool,
        owner_id: Uuid,
        filter: MailboxEntryFilterDto,
    ) -> Result<Vec<MailboxEntry>, PoolError>;

    async fn find_by_owner_and_id(
        &self,
        pool: Pool,
        owner_id: Uuid,
        mailbox_entry_id: Uuid,
    ) -> Result<Option<MailboxEntry>, PoolError>;

    async fn mark_as_read(
        &self,
        tx: &mut Transaction<'_>,
        owner_id: Uuid,
        mailbox_entry_id: Uuid,
    ) -> Result<Option<MailboxEntry>, PoolError>;

    async fn create_booking_request_entries(
        &self,
        tx: &mut Transaction<'_>,
        payload: CreateBookingRequestMailboxEntriesDto,
    ) -> Result<(), PoolError>;
}
