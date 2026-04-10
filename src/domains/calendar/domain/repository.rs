use async_trait::async_trait;
use deadpool_postgres::{Pool, PoolError, Transaction};
use uuid::Uuid;

use crate::domains::calendar::{
    domain::model::CalendarEntry,
    dto::calendar_dto::{CalendarEntryFilterDto, CreateCalendarEntryDto, UpdateCalendarEntryDto},
};

#[async_trait]
pub trait CalendarRepository: Send + Sync {
    async fn article_exists(&self, pool: Pool, article_id: Uuid) -> Result<bool, PoolError>;

    async fn find_by_article_id(
        &self,
        pool: Pool,
        article_id: Uuid,
        filter: CalendarEntryFilterDto,
    ) -> Result<Vec<CalendarEntry>, PoolError>;

    async fn find_by_id(
        &self,
        pool: Pool,
        article_id: Uuid,
        event_id: Uuid,
    ) -> Result<Option<CalendarEntry>, PoolError>;

    async fn create(
        &self,
        tx: &mut Transaction<'_>,
        event_id: Uuid,
        article_id: Uuid,
        payload: CreateCalendarEntryDto,
    ) -> Result<(), PoolError>;

    async fn update(
        &self,
        tx: &mut Transaction<'_>,
        article_id: Uuid,
        event_id: Uuid,
        payload: UpdateCalendarEntryDto,
    ) -> Result<Option<CalendarEntry>, PoolError>;

    async fn delete(
        &self,
        tx: &mut Transaction<'_>,
        article_id: Uuid,
        event_id: Uuid,
    ) -> Result<bool, PoolError>;
}
