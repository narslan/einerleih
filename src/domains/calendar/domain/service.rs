use std::sync::Arc;

use async_trait::async_trait;
use deadpool_postgres::Pool;
use uuid::Uuid;

use crate::{
    common::error::AppError,
    domains::calendar::dto::calendar_dto::{
        CalendarEntryDto, CalendarEntryFilterDto, CreateCalendarEntryDto, UpdateCalendarEntryDto,
    },
};

#[async_trait]
pub trait CalendarServiceTrait: Send + Sync {
    fn create_service(pool: Pool) -> Arc<dyn CalendarServiceTrait>
    where
        Self: Sized;

    async fn get_entries_for_article(
        &self,
        article_id: Uuid,
        filter: CalendarEntryFilterDto,
    ) -> Result<Vec<CalendarEntryDto>, AppError>;

    async fn get_entry(
        &self,
        article_id: Uuid,
        event_id: Uuid,
    ) -> Result<CalendarEntryDto, AppError>;

    async fn create_entry(
        &self,
        article_id: Uuid,
        payload: CreateCalendarEntryDto,
    ) -> Result<CalendarEntryDto, AppError>;

    async fn update_entry(
        &self,
        article_id: Uuid,
        event_id: Uuid,
        payload: UpdateCalendarEntryDto,
    ) -> Result<CalendarEntryDto, AppError>;

    async fn delete_entry(&self, article_id: Uuid, event_id: Uuid) -> Result<String, AppError>;
}
