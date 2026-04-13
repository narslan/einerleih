use async_trait::async_trait;
use deadpool_postgres::{Pool, PoolError, Transaction};
use tokio_postgres::Row;
use uuid::Uuid;

use crate::domains::calendar::{
    domain::{
        model::{CalendarEntry, CalendarEntrySource},
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
        start_date,
        end_date,
        location,
        description,
        dtstamp,
        source,
        created_by,
        created_at,
        modified_by,
        modified_at
    FROM event_calendar
    WHERE article_id = $1
        AND ($2::DATE IS NULL OR end_date >= $2)
        AND ($3::DATE IS NULL OR start_date <= $3)
    ORDER BY start_date ASC, end_date ASC, event_id ASC
    "#;

const FIND_BY_ID_QUERY: &str = r#"
    SELECT
        event_id,
        article_id,
        start_date,
        end_date,
        location,
        description,
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
        start_date,
        end_date,
        location,
        description,
        source,
        created_by,
        modified_by
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
    "#;

const UPDATE_ENTRY_QUERY: &str = r#"
    UPDATE event_calendar
    SET
        start_date = $3,
        end_date = $4,
        location = $5,
        description = $6,
        source = $7,
        modified_by = $8,
        modified_at = NOW()
    WHERE article_id = $1
        AND event_id = $2
    RETURNING
        event_id,
        article_id,
        start_date,
        end_date,
        location,
        description,
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
    let source: String = row.get(7);

    CalendarEntry {
        event_id: row.get(0),
        article_id: row.get(1),
        start_date: row.get(2),
        end_date: row.get(3),
        location: row.get(4),
        description: row.get(5),
        dtstamp: row.get(6),
        source: CalendarEntrySource::from_db(&source)
            .unwrap_or_else(|| panic!("invalid calendar entry source in database: {source}")),
        created_by: row.get(8),
        created_at: row.get(9),
        modified_by: row.get(10),
        modified_at: row.get(11),
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
        let rows = client
            .query(&stmt, &[&article_id, &filter.start_date, &filter.end_date])
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
        let stmt = tx.prepare_cached(CREATE_ENTRY_QUERY).await?;
        tx.execute(
            &stmt,
            &[
                &event_id,
                &article_id,
                &payload.start_date,
                &payload.end_date,
                &payload.location,
                &payload.description,
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
        let stmt = tx.prepare_cached(UPDATE_ENTRY_QUERY).await?;
        let row = tx
            .query_opt(
                &stmt,
                &[
                    &article_id,
                    &event_id,
                    &payload.start_date,
                    &payload.end_date,
                    &payload.location,
                    &payload.description,
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
