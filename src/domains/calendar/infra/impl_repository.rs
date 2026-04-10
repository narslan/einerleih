use async_trait::async_trait;
use chrono::{DateTime, Utc};
use deadpool_postgres::{Pool, PoolError, Transaction};
use tokio_postgres::Row;
use uuid::Uuid;

use crate::domains::calendar::{
    domain::{
        model::{CalendarBlockReason, CalendarEntry, CalendarEntrySource, CalendarEntryType},
        repository::CalendarRepository,
    },
    dto::calendar_dto::{CalendarEntryFilterDto, CreateCalendarEntryDto, UpdateCalendarEntryDto},
};

pub struct CalendarRepo;

const ARTICLE_EXISTS_QUERY: &str = r#"
    SELECT EXISTS(
        SELECT 1
        FROM article
        WHERE article_id = $1
    )
    "#;

const FIND_BY_ARTICLE_ID_QUERY: &str = r#"
    SELECT
        event_id,
        article_id,
        entry_type,
        block_reason,
        summary,
        location,
        description,
        start_time,
        end_time,
        rrule,
        dtstamp,
        source,
        created_by,
        created_at,
        modified_by,
        modified_at
    FROM event_calendar
    WHERE article_id = $1
        AND ($2::TIMESTAMPTZ IS NULL OR end_time > $2)
        AND ($3::TIMESTAMPTZ IS NULL OR start_time < $3)
    ORDER BY start_time ASC, end_time ASC, event_id ASC
    "#;

const FIND_BY_ID_QUERY: &str = r#"
    SELECT
        event_id,
        article_id,
        entry_type,
        block_reason,
        summary,
        location,
        description,
        start_time,
        end_time,
        rrule,
        dtstamp,
        source,
        created_by,
        created_at,
        modified_by,
        modified_at
    FROM event_calendar
    WHERE article_id = $1
        AND event_id = $2
    "#;

const CREATE_ENTRY_QUERY: &str = r#"
    INSERT INTO event_calendar (
        event_id,
        article_id,
        entry_type,
        block_reason,
        summary,
        location,
        description,
        start_time,
        end_time,
        rrule,
        source,
        created_by,
        modified_by
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
    "#;

const UPDATE_ENTRY_QUERY: &str = r#"
    UPDATE event_calendar
    SET
        entry_type = $3,
        block_reason = $4,
        summary = $5,
        location = $6,
        description = $7,
        start_time = $8,
        end_time = $9,
        rrule = $10,
        source = $11,
        modified_by = $12,
        modified_at = NOW()
    WHERE article_id = $1
        AND event_id = $2
    RETURNING
        event_id,
        article_id,
        entry_type,
        block_reason,
        summary,
        location,
        description,
        start_time,
        end_time,
        rrule,
        dtstamp,
        source,
        created_by,
        created_at,
        modified_by,
        modified_at
    "#;

const DELETE_ENTRY_QUERY: &str = r#"
    DELETE FROM event_calendar
    WHERE article_id = $1
        AND event_id = $2
    "#;

fn map_calendar_entry_row(row: &Row) -> CalendarEntry {
    let entry_type: String = row.get(2);
    let block_reason: Option<String> = row.get(3);
    let source: String = row.get(11);

    CalendarEntry {
        event_id: row.get(0),
        article_id: row.get(1),
        entry_type: CalendarEntryType::from_db(&entry_type)
            .unwrap_or_else(|| panic!("invalid calendar entry type in database: {entry_type}")),
        block_reason: block_reason.map(|value| {
            CalendarBlockReason::from_db(&value)
                .unwrap_or_else(|| panic!("invalid calendar block reason in database: {value}"))
        }),
        summary: row.get(4),
        location: row.get(5),
        description: row.get(6),
        start_time: row.get(7),
        end_time: row.get(8),
        rrule: row.get(9),
        dtstamp: row.get(10),
        source: CalendarEntrySource::from_db(&source)
            .unwrap_or_else(|| panic!("invalid calendar entry source in database: {source}")),
        created_by: row.get(12),
        created_at: row.get(13),
        modified_by: row.get(14),
        modified_at: row.get(15),
    }
}

#[async_trait]
impl CalendarRepository for CalendarRepo {
    async fn article_exists(&self, pool: Pool, article_id: Uuid) -> Result<bool, PoolError> {
        let client = pool.get().await?;
        let stmt = client.prepare_cached(ARTICLE_EXISTS_QUERY).await?;
        let row = client.query_one(&stmt, &[&article_id]).await?;
        Ok(row.get(0))
    }

    async fn find_by_article_id(
        &self,
        pool: Pool,
        article_id: Uuid,
        filter: CalendarEntryFilterDto,
    ) -> Result<Vec<CalendarEntry>, PoolError> {
        let client = pool.get().await?;
        let stmt = client.prepare_cached(FIND_BY_ARTICLE_ID_QUERY).await?;
        let start_time: Option<DateTime<Utc>> = filter.start_time;
        let end_time: Option<DateTime<Utc>> = filter.end_time;
        let rows = client
            .query(&stmt, &[&article_id, &start_time, &end_time])
            .await?;
        Ok(rows
            .into_iter()
            .map(|row| map_calendar_entry_row(&row))
            .collect())
    }

    async fn find_by_id(
        &self,
        pool: Pool,
        article_id: Uuid,
        event_id: Uuid,
    ) -> Result<Option<CalendarEntry>, PoolError> {
        let client = pool.get().await?;
        let stmt = client.prepare_cached(FIND_BY_ID_QUERY).await?;
        let row = client.query_opt(&stmt, &[&article_id, &event_id]).await?;
        Ok(row.map(|row| map_calendar_entry_row(&row)))
    }

    async fn create(
        &self,
        tx: &mut Transaction<'_>,
        event_id: Uuid,
        article_id: Uuid,
        payload: CreateCalendarEntryDto,
    ) -> Result<(), PoolError> {
        let block_reason = payload
            .block_reason
            .as_ref()
            .map(CalendarBlockReason::as_str);
        let stmt = tx.prepare_cached(CREATE_ENTRY_QUERY).await?;
        tx.execute(
            &stmt,
            &[
                &event_id,
                &article_id,
                &payload.entry_type.as_str(),
                &block_reason,
                &payload.summary,
                &payload.location,
                &payload.description,
                &payload.start_time,
                &payload.end_time,
                &payload.rrule,
                &payload.source.as_str(),
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
        event_id: Uuid,
        payload: UpdateCalendarEntryDto,
    ) -> Result<Option<CalendarEntry>, PoolError> {
        let block_reason = payload
            .block_reason
            .as_ref()
            .map(CalendarBlockReason::as_str);
        let stmt = tx.prepare_cached(UPDATE_ENTRY_QUERY).await?;
        let row = tx
            .query_opt(
                &stmt,
                &[
                    &article_id,
                    &event_id,
                    &payload.entry_type.as_str(),
                    &block_reason,
                    &payload.summary,
                    &payload.location,
                    &payload.description,
                    &payload.start_time,
                    &payload.end_time,
                    &payload.rrule,
                    &payload.source.as_str(),
                    &payload.modified_by,
                ],
            )
            .await?;

        Ok(row.map(|row| map_calendar_entry_row(&row)))
    }

    async fn delete(
        &self,
        tx: &mut Transaction<'_>,
        article_id: Uuid,
        event_id: Uuid,
    ) -> Result<bool, PoolError> {
        let stmt = tx.prepare_cached(DELETE_ENTRY_QUERY).await?;
        let affected_rows = tx.execute(&stmt, &[&article_id, &event_id]).await?;
        Ok(affected_rows > 0)
    }
}
