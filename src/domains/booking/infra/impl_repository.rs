use async_trait::async_trait;
use chrono::{DateTime, Utc};
use deadpool_postgres::{Pool, PoolError, Transaction};
use tokio_postgres::Row;
use uuid::Uuid;

use crate::domains::booking::{
    domain::{
        model::{Booking, BookingStatus},
        repository::BookingRepository,
    },
    dto::booking_dto::{BookingFilterDto, CreateBookingDto, UpdateBookingDto},
};

pub struct BookingRepo;

const ARTICLE_EXISTS_QUERY: &str = r#"
    SELECT EXISTS(
        SELECT 1
        FROM article
        WHERE article_id = $1
    )
    "#;

const CONFIRMED_BOOKING_CONFLICT_QUERY: &str = r#"
    SELECT EXISTS(
        SELECT 1
        FROM booking
        WHERE article_id = $1
            AND status = 'confirmed'
            AND ($2::UUID IS NULL OR booking_id <> $2)
            AND tstzrange(start_time, end_time, '[)') && tstzrange($3::TIMESTAMPTZ, $4::TIMESTAMPTZ, '[)')
    )
    "#;

const CALENDAR_BLOCK_CONFLICT_QUERY: &str = r#"
    SELECT EXISTS(
        SELECT 1
        FROM event_calendar
        WHERE article_id = $1
            AND entry_type = 'block'
            AND tstzrange(start_time, end_time, '[)') && tstzrange($2::TIMESTAMPTZ, $3::TIMESTAMPTZ, '[)')
    )
    "#;

const FIND_BY_ARTICLE_ID_QUERY: &str = r#"
    SELECT
        booking_id,
        article_id,
        requested_by,
        requester_name,
        requester_email,
        note,
        start_time,
        end_time,
        status,
        approved_by,
        approved_at,
        created_by,
        created_at,
        modified_by,
        modified_at
    FROM booking
    WHERE article_id = $1
        AND ($2::TIMESTAMPTZ IS NULL OR end_time > $2)
        AND ($3::TIMESTAMPTZ IS NULL OR start_time < $3)
        AND ($4::VARCHAR IS NULL OR status = $4)
    ORDER BY start_time ASC, end_time ASC, booking_id ASC
    "#;

const FIND_BY_ID_QUERY: &str = r#"
    SELECT
        booking_id,
        article_id,
        requested_by,
        requester_name,
        requester_email,
        note,
        start_time,
        end_time,
        status,
        approved_by,
        approved_at,
        created_by,
        created_at,
        modified_by,
        modified_at
    FROM booking
    WHERE article_id = $1
        AND booking_id = $2
    "#;

const CREATE_BOOKING_QUERY: &str = r#"
    INSERT INTO booking (
        booking_id,
        article_id,
        requested_by,
        requester_name,
        requester_email,
        note,
        start_time,
        end_time,
        status,
        created_by,
        modified_by
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'requested', $9, $10)
    "#;

const UPDATE_BOOKING_QUERY: &str = r#"
    UPDATE booking
    SET
        requested_by = $3,
        requester_name = $4,
        requester_email = $5,
        note = $6,
        start_time = $7,
        end_time = $8,
        modified_by = $9,
        modified_at = NOW()
    WHERE article_id = $1
        AND booking_id = $2
    RETURNING
        booking_id,
        article_id,
        requested_by,
        requester_name,
        requester_email,
        note,
        start_time,
        end_time,
        status,
        approved_by,
        approved_at,
        created_by,
        created_at,
        modified_by,
        modified_at
    "#;

const UPDATE_STATUS_QUERY: &str = r#"
    UPDATE booking
    SET
        status = $3::VARCHAR,
        approved_by = CASE WHEN $3::VARCHAR = 'confirmed' THEN $4 ELSE approved_by END,
        approved_at = CASE WHEN $3::VARCHAR = 'confirmed' THEN NOW() ELSE approved_at END,
        modified_by = $4,
        modified_at = NOW()
    WHERE article_id = $1
        AND booking_id = $2
    RETURNING
        booking_id,
        article_id,
        requested_by,
        requester_name,
        requester_email,
        note,
        start_time,
        end_time,
        status,
        approved_by,
        approved_at,
        created_by,
        created_at,
        modified_by,
        modified_at
    "#;

fn map_booking_row(row: &Row) -> Booking {
    let status: String = row.get(8);

    Booking {
        booking_id: row.get(0),
        article_id: row.get(1),
        requested_by: row.get(2),
        requester_name: row.get(3),
        requester_email: row.get(4),
        note: row.get(5),
        start_time: row.get(6),
        end_time: row.get(7),
        status: BookingStatus::from_db(&status)
            .unwrap_or_else(|| panic!("invalid booking status in database: {status}")),
        approved_by: row.get(9),
        approved_at: row.get(10),
        created_by: row.get(11),
        created_at: row.get(12),
        modified_by: row.get(13),
        modified_at: row.get(14),
    }
}

#[async_trait]
impl BookingRepository for BookingRepo {
    async fn article_exists(&self, pool: Pool, article_id: Uuid) -> Result<bool, PoolError> {
        let client = pool.get().await?;
        let stmt = client.prepare_cached(ARTICLE_EXISTS_QUERY).await?;
        let row = client.query_one(&stmt, &[&article_id]).await?;
        Ok(row.get(0))
    }

    async fn has_confirmed_booking_conflict(
        &self,
        pool: Pool,
        article_id: Uuid,
        booking_id: Option<Uuid>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<bool, PoolError> {
        let client = pool.get().await?;
        let stmt = client
            .prepare_cached(CONFIRMED_BOOKING_CONFLICT_QUERY)
            .await?;
        let row = client
            .query_one(&stmt, &[&article_id, &booking_id, &start_time, &end_time])
            .await?;
        Ok(row.get(0))
    }

    async fn has_calendar_block_conflict(
        &self,
        pool: Pool,
        article_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<bool, PoolError> {
        let client = pool.get().await?;
        let stmt = client.prepare_cached(CALENDAR_BLOCK_CONFLICT_QUERY).await?;
        let row = client
            .query_one(&stmt, &[&article_id, &start_time, &end_time])
            .await?;
        Ok(row.get(0))
    }

    async fn find_by_article_id(
        &self,
        pool: Pool,
        article_id: Uuid,
        filter: BookingFilterDto,
    ) -> Result<Vec<Booking>, PoolError> {
        let client = pool.get().await?;
        let stmt = client.prepare_cached(FIND_BY_ARTICLE_ID_QUERY).await?;
        let start_time: Option<DateTime<Utc>> = filter.start_time;
        let end_time: Option<DateTime<Utc>> = filter.end_time;
        let status = filter.status.as_ref().map(BookingStatus::as_str);
        let rows = client
            .query(&stmt, &[&article_id, &start_time, &end_time, &status])
            .await?;
        Ok(rows.into_iter().map(|row| map_booking_row(&row)).collect())
    }

    async fn find_by_id(
        &self,
        pool: Pool,
        article_id: Uuid,
        booking_id: Uuid,
    ) -> Result<Option<Booking>, PoolError> {
        let client = pool.get().await?;
        let stmt = client.prepare_cached(FIND_BY_ID_QUERY).await?;
        let row = client.query_opt(&stmt, &[&article_id, &booking_id]).await?;
        Ok(row.map(|row| map_booking_row(&row)))
    }

    async fn create(
        &self,
        tx: &mut Transaction<'_>,
        booking_id: Uuid,
        article_id: Uuid,
        payload: CreateBookingDto,
    ) -> Result<(), PoolError> {
        let stmt = tx.prepare_cached(CREATE_BOOKING_QUERY).await?;
        tx.execute(
            &stmt,
            &[
                &booking_id,
                &article_id,
                &payload.requested_by,
                &payload.requester_name,
                &payload.requester_email,
                &payload.note,
                &payload.start_time,
                &payload.end_time,
                &payload.created_by,
                &payload.modified_by,
            ],
        )
        .await?;
        Ok(())
    }

    async fn update(
        &self,
        tx: &mut Transaction<'_>,
        article_id: Uuid,
        booking_id: Uuid,
        payload: UpdateBookingDto,
    ) -> Result<Option<Booking>, PoolError> {
        let stmt = tx.prepare_cached(UPDATE_BOOKING_QUERY).await?;
        let row = tx
            .query_opt(
                &stmt,
                &[
                    &article_id,
                    &booking_id,
                    &payload.requested_by,
                    &payload.requester_name,
                    &payload.requester_email,
                    &payload.note,
                    &payload.start_time,
                    &payload.end_time,
                    &payload.modified_by,
                ],
            )
            .await?;

        Ok(row.map(|row| map_booking_row(&row)))
    }

    async fn update_status(
        &self,
        tx: &mut Transaction<'_>,
        article_id: Uuid,
        booking_id: Uuid,
        status: BookingStatus,
        actor_id: Uuid,
    ) -> Result<Option<Booking>, PoolError> {
        let stmt = tx.prepare_cached(UPDATE_STATUS_QUERY).await?;
        let row = tx
            .query_opt(
                &stmt,
                &[&article_id, &booking_id, &status.as_str(), &actor_id],
            )
            .await?;

        Ok(row.map(|row| map_booking_row(&row)))
    }
}
