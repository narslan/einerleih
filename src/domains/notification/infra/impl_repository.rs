use async_trait::async_trait;
use deadpool_postgres::{Pool, PoolError, Transaction};
use tokio_postgres::Row;
use uuid::Uuid;

use crate::domains::notification::{
    NotificationKind, NotificationOutboxEntry, NotificationOutboxRepository, NotificationStatus,
    dto::notification_dto::{CreateBookingRequestNotificationDto, EnqueueEmailNotificationDto},
};

pub struct NotificationOutboxRepo;

const INSERT_NOTIFICATION_QUERY: &str = r#"
    INSERT INTO notification_outbox (
        notification_id,
        kind,
        status,
        recipient_email,
        subject,
        body_text,
        booking_id,
        article_id,
        user_id,
        created_by
    )
    VALUES ($1, $2, 'pending', $3, $4, $5, $6, $7, $8, $9)
    "#;

const FIND_DISPATCHABLE_QUERY: &str = r#"
    SELECT
        notification_id,
        kind,
        status,
        recipient_email,
        subject,
        body_text,
        booking_id,
        article_id,
        user_id,
        attempt_count,
        last_error,
        created_by,
        created_at,
        sent_at
    FROM notification_outbox
    WHERE status IN ('pending', 'failed')
    ORDER BY created_at ASC, notification_id ASC
    LIMIT $1
    "#;

const MARK_SENT_QUERY: &str = r#"
    UPDATE notification_outbox
    SET
        status = 'sent',
        attempt_count = attempt_count + 1,
        last_error = NULL,
        sent_at = NOW()
    WHERE notification_id = $1
    "#;

const MARK_FAILED_QUERY: &str = r#"
    UPDATE notification_outbox
    SET
        status = 'failed',
        attempt_count = attempt_count + 1,
        last_error = $2
    WHERE notification_id = $1
    "#;

const BOOKING_NOTIFICATION_CONTEXT_QUERY: &str = r#"
    SELECT
        a.name,
        a.created_by,
        u.email
    FROM article a
    LEFT JOIN users u ON u.id = a.created_by
    WHERE a.article_id = $1
    "#;

fn map_outbox_row(row: &Row) -> NotificationOutboxEntry {
    let kind: String = row.get(1);
    let status: String = row.get(2);

    NotificationOutboxEntry {
        notification_id: row.get(0),
        kind: NotificationKind::from_db(&kind)
            .unwrap_or_else(|| panic!("invalid notification kind in database: {kind}")),
        status: NotificationStatus::from_db(&status)
            .unwrap_or_else(|| panic!("invalid notification status in database: {status}")),
        recipient_email: row.get(3),
        subject: row.get(4),
        body_text: row.get(5),
        booking_id: row.get(6),
        article_id: row.get(7),
        user_id: row.get(8),
        attempt_count: row.get(9),
        last_error: row.get(10),
        created_by: row.get(11),
        created_at: row.get(12),
        sent_at: row.get(13),
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
            format!(
                "{requester} hat eine Buchungsanfrage fuer \"{article_name}\" gesendet.\n\nNachricht:\n{note}"
            )
        }
        None => format!("{requester} hat eine Buchungsanfrage fuer \"{article_name}\" gesendet."),
    }
}

#[async_trait]
impl NotificationOutboxRepository for NotificationOutboxRepo {
    async fn enqueue(
        &self,
        tx: &mut Transaction<'_>,
        payload: EnqueueEmailNotificationDto,
    ) -> Result<Uuid, PoolError> {
        let notification_id = Uuid::new_v4();
        let stmt = tx.prepare_cached(INSERT_NOTIFICATION_QUERY).await?;
        tx.execute(
            &stmt,
            &[
                &notification_id,
                &payload.kind.as_str(),
                &payload.recipient_email,
                &payload.subject,
                &payload.body_text,
                &payload.booking_id,
                &payload.article_id,
                &payload.user_id,
                &payload.created_by,
            ],
        )
        .await?;
        Ok(notification_id)
    }

    async fn enqueue_booking_request(
        &self,
        tx: &mut Transaction<'_>,
        payload: CreateBookingRequestNotificationDto,
    ) -> Result<Option<Uuid>, PoolError> {
        let context_stmt = tx
            .prepare_cached(BOOKING_NOTIFICATION_CONTEXT_QUERY)
            .await?;
        let Some(context) = tx.query_opt(&context_stmt, &[&payload.article_id]).await? else {
            return Ok(None);
        };

        let article_name: String = context.get(0);
        let provider_id: Option<Uuid> = context.get(1);
        let provider_email: Option<String> = context.get(2);

        let Some(provider_id) = provider_id else {
            return Ok(None);
        };

        let Some(provider_email) = provider_email.filter(|email| !email.trim().is_empty()) else {
            return Ok(None);
        };

        let body_text = build_booking_request_body(
            payload.requester_name.as_deref(),
            payload.note.as_deref(),
            &article_name,
        );
        let subject = format!("Neue Anfrage fuer {article_name}");

        let notification_id = self
            .enqueue(
                tx,
                EnqueueEmailNotificationDto {
                    kind: NotificationKind::BookingRequest,
                    recipient_email: provider_email,
                    subject,
                    body_text,
                    booking_id: Some(payload.booking_id),
                    article_id: Some(payload.article_id),
                    user_id: Some(provider_id),
                    created_by: payload.created_by,
                },
            )
            .await?;

        Ok(Some(notification_id))
    }

    async fn find_dispatchable(
        &self,
        pool: Pool,
        limit: i64,
    ) -> Result<Vec<NotificationOutboxEntry>, PoolError> {
        let client = pool.get().await?;
        let stmt = client.prepare_cached(FIND_DISPATCHABLE_QUERY).await?;
        let rows = client.query(&stmt, &[&limit]).await?;
        Ok(rows.into_iter().map(|row| map_outbox_row(&row)).collect())
    }

    async fn mark_sent(
        &self,
        tx: &mut Transaction<'_>,
        notification_id: Uuid,
    ) -> Result<(), PoolError> {
        let stmt = tx.prepare_cached(MARK_SENT_QUERY).await?;
        tx.execute(&stmt, &[&notification_id]).await?;
        Ok(())
    }

    async fn mark_failed(
        &self,
        tx: &mut Transaction<'_>,
        notification_id: Uuid,
        error_message: &str,
    ) -> Result<(), PoolError> {
        let stmt = tx.prepare_cached(MARK_FAILED_QUERY).await?;
        tx.execute(&stmt, &[&notification_id, &error_message])
            .await?;
        Ok(())
    }
}
