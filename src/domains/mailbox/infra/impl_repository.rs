use async_trait::async_trait;
use deadpool_postgres::{Pool, PoolError, Transaction};
use tokio_postgres::Row;
use uuid::Uuid;

use crate::domains::mailbox::{
    domain::{
        model::{MailboxDirection, MailboxEntry},
        repository::MailboxRepository,
    },
    dto::mailbox_dto::{CreateBookingRequestMailboxEntriesDto, MailboxEntryFilterDto},
};

pub struct MailboxRepo;

const FIND_BY_OWNER_ID_QUERY: &str = r#"
    SELECT
        mailbox_entry_id,
        booking_id,
        article_id,
        owner_id,
        sender_id,
        recipient_id,
        direction,
        subject,
        body,
        read_at,
        created_by,
        created_at,
        modified_by,
        modified_at
    FROM mailbox_entry
    WHERE owner_id = $1
        AND ($2::VARCHAR IS NULL OR direction = $2)
    ORDER BY created_at DESC, mailbox_entry_id DESC
    "#;

const FIND_BY_OWNER_AND_ID_QUERY: &str = r#"
    SELECT
        mailbox_entry_id,
        booking_id,
        article_id,
        owner_id,
        sender_id,
        recipient_id,
        direction,
        subject,
        body,
        read_at,
        created_by,
        created_at,
        modified_by,
        modified_at
    FROM mailbox_entry
    WHERE owner_id = $1
        AND mailbox_entry_id = $2
    "#;

const MARK_AS_READ_QUERY: &str = r#"
    UPDATE mailbox_entry
    SET
        read_at = COALESCE(read_at, NOW()),
        modified_by = $1,
        modified_at = NOW()
    WHERE owner_id = $1
        AND mailbox_entry_id = $2
    RETURNING
        mailbox_entry_id,
        booking_id,
        article_id,
        owner_id,
        sender_id,
        recipient_id,
        direction,
        subject,
        body,
        read_at,
        created_by,
        created_at,
        modified_by,
        modified_at
    "#;

const ARTICLE_MAILBOX_CONTEXT_QUERY: &str = r#"
    SELECT name, created_by
    FROM article
    WHERE article_id = $1
    "#;

const CREATE_MAILBOX_ENTRY_QUERY: &str = r#"
    INSERT INTO mailbox_entry (
        mailbox_entry_id,
        booking_id,
        article_id,
        owner_id,
        sender_id,
        recipient_id,
        direction,
        subject,
        body,
        created_by,
        modified_by
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
    "#;

fn map_mailbox_entry_row(row: &Row) -> MailboxEntry {
    let direction: String = row.get(6);

    MailboxEntry {
        mailbox_entry_id: row.get(0),
        booking_id: row.get(1),
        article_id: row.get(2),
        owner_id: row.get(3),
        sender_id: row.get(4),
        recipient_id: row.get(5),
        direction: MailboxDirection::from_db(&direction)
            .unwrap_or_else(|| panic!("invalid mailbox direction in database: {direction}")),
        subject: row.get(7),
        body: row.get(8),
        read_at: row.get(9),
        created_by: row.get(10),
        created_at: row.get(11),
        modified_by: row.get(12),
        modified_at: row.get(13),
    }
}

fn build_booking_request_body(
    requester_name: Option<&str>,
    note: Option<&str>,
    article_name: &str,
) -> String {
    let requester = requester_name
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Ein Nutzer");
    let note = note.filter(|value| !value.trim().is_empty());

    match note {
        Some(note) => {
            format!("{requester} hat eine Anfrage fuer \"{article_name}\" gesendet.\n\n{note}")
        }
        None => format!("{requester} hat eine Anfrage fuer \"{article_name}\" gesendet."),
    }
}

#[async_trait]
impl MailboxRepository for MailboxRepo {
    async fn find_by_owner_id(
        &self,
        pool: Pool,
        owner_id: Uuid,
        filter: MailboxEntryFilterDto,
    ) -> Result<Vec<MailboxEntry>, PoolError> {
        let client = pool.get().await?;
        let stmt = client.prepare_cached(FIND_BY_OWNER_ID_QUERY).await?;
        let direction = filter.direction.as_ref().map(MailboxDirection::as_str);
        let rows = client.query(&stmt, &[&owner_id, &direction]).await?;
        Ok(rows
            .into_iter()
            .map(|row| map_mailbox_entry_row(&row))
            .collect())
    }

    async fn find_by_owner_and_id(
        &self,
        pool: Pool,
        owner_id: Uuid,
        mailbox_entry_id: Uuid,
    ) -> Result<Option<MailboxEntry>, PoolError> {
        let client = pool.get().await?;
        let stmt = client.prepare_cached(FIND_BY_OWNER_AND_ID_QUERY).await?;
        let row = client
            .query_opt(&stmt, &[&owner_id, &mailbox_entry_id])
            .await?;
        Ok(row.map(|row| map_mailbox_entry_row(&row)))
    }

    async fn mark_as_read(
        &self,
        tx: &mut Transaction<'_>,
        owner_id: Uuid,
        mailbox_entry_id: Uuid,
    ) -> Result<Option<MailboxEntry>, PoolError> {
        let stmt = tx.prepare_cached(MARK_AS_READ_QUERY).await?;
        let row = tx.query_opt(&stmt, &[&owner_id, &mailbox_entry_id]).await?;
        Ok(row.map(|row| map_mailbox_entry_row(&row)))
    }

    async fn create_booking_request_entries(
        &self,
        tx: &mut Transaction<'_>,
        payload: CreateBookingRequestMailboxEntriesDto,
    ) -> Result<(), PoolError> {
        let article_context_stmt = tx.prepare_cached(ARTICLE_MAILBOX_CONTEXT_QUERY).await?;
        let Some(article_context) = tx
            .query_opt(&article_context_stmt, &[&payload.article_id])
            .await?
        else {
            return Ok(());
        };
        let article_name: String = article_context.get(0);
        let provider_id: Option<Uuid> = article_context.get(1);
        let Some(provider_id) = provider_id else {
            return Ok(());
        };

        let subject = format!("Anfrage fuer {article_name}");
        let body = build_booking_request_body(
            payload.requester_name.as_deref(),
            payload.note.as_deref(),
            &article_name,
        );
        let insert_stmt = tx.prepare_cached(CREATE_MAILBOX_ENTRY_QUERY).await?;

        tx.execute(
            &insert_stmt,
            &[
                &Uuid::new_v4(),
                &Some(payload.booking_id),
                &payload.article_id,
                &payload.requester_id,
                &payload.requester_id,
                &provider_id,
                &MailboxDirection::Sent.as_str(),
                &subject,
                &body,
                &payload.created_by,
                &payload.created_by,
            ],
        )
        .await?;

        tx.execute(
            &insert_stmt,
            &[
                &Uuid::new_v4(),
                &Some(payload.booking_id),
                &payload.article_id,
                &provider_id,
                &payload.requester_id,
                &provider_id,
                &MailboxDirection::Inbox.as_str(),
                &subject,
                &body,
                &payload.created_by,
                &payload.created_by,
            ],
        )
        .await?;

        Ok(())
    }
}
