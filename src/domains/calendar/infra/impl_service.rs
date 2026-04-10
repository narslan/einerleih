use std::sync::Arc;

use async_trait::async_trait;
use deadpool_postgres::Pool;
use uuid::Uuid;

use crate::{
    common::error::AppError,
    domains::calendar::{
        domain::{
            model::{CalendarBlockReason, CalendarEntryType},
            repository::CalendarRepository,
            service::CalendarServiceTrait,
        },
        dto::calendar_dto::{
            CalendarEntryDto, CalendarEntryFilterDto, CreateCalendarEntryDto,
            UpdateCalendarEntryDto,
        },
        infra::impl_repository::CalendarRepo,
    },
};

#[derive(Clone)]
pub struct CalendarService {
    pub pool: Pool,
    pub repo: Arc<dyn CalendarRepository + Send + Sync>,
}

fn validate_entry_semantics(
    entry_type: &CalendarEntryType,
    block_reason: &Option<CalendarBlockReason>,
) -> Result<(), AppError> {
    match (entry_type, block_reason) {
        (CalendarEntryType::Availability, None) => Ok(()),
        (CalendarEntryType::Availability, Some(_)) => Err(AppError::ValidationError(
            "block_reason must be empty for availability entries".into(),
        )),
        (CalendarEntryType::Block, Some(_)) => Ok(()),
        (CalendarEntryType::Block, None) => Err(AppError::ValidationError(
            "block_reason is required for block entries".into(),
        )),
    }
}

fn validate_time_order(
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: chrono::DateTime<chrono::Utc>,
) -> Result<(), AppError> {
    if end_time <= start_time {
        return Err(AppError::ValidationError(
            "end_time must be after start_time".into(),
        ));
    }

    Ok(())
}

#[async_trait]
impl CalendarServiceTrait for CalendarService {
    fn create_service(pool: Pool) -> Arc<dyn CalendarServiceTrait> {
        Arc::new(Self {
            pool,
            repo: Arc::new(CalendarRepo {}),
        })
    }

    async fn get_entries_for_article(
        &self,
        article_id: Uuid,
        filter: CalendarEntryFilterDto,
    ) -> Result<Vec<CalendarEntryDto>, AppError> {
        if !self
            .repo
            .article_exists(self.pool.clone(), article_id)
            .await
            .map_err(AppError::DatabaseError)?
        {
            return Err(AppError::NotFound("Article not found".into()));
        }

        if let (Some(start_time), Some(end_time)) = (filter.start_time, filter.end_time) {
            validate_time_order(start_time, end_time)?;
        }

        self.repo
            .find_by_article_id(self.pool.clone(), article_id, filter)
            .await
            .map(|entries| entries.into_iter().map(CalendarEntryDto::from).collect())
            .map_err(AppError::DatabaseError)
    }

    async fn get_entry(
        &self,
        article_id: Uuid,
        event_id: Uuid,
    ) -> Result<CalendarEntryDto, AppError> {
        self.repo
            .find_by_id(self.pool.clone(), article_id, event_id)
            .await
            .map_err(AppError::DatabaseError)?
            .map(CalendarEntryDto::from)
            .ok_or_else(|| AppError::NotFound("Calendar entry not found".into()))
    }

    async fn create_entry(
        &self,
        article_id: Uuid,
        payload: CreateCalendarEntryDto,
    ) -> Result<CalendarEntryDto, AppError> {
        validate_entry_semantics(&payload.entry_type, &payload.block_reason)?;
        validate_time_order(payload.start_time, payload.end_time)?;

        if !self
            .repo
            .article_exists(self.pool.clone(), article_id)
            .await
            .map_err(AppError::DatabaseError)?
        {
            return Err(AppError::NotFound("Article not found".into()));
        }

        let mut client = self.pool.get().await.map_err(AppError::DatabaseError)?;
        let mut tx = client.transaction().await.map_err(|err| {
            tracing::error!("Error starting calendar entry creation transaction: {err}");
            AppError::InternalError
        })?;
        let event_id = Uuid::new_v4();

        if let Err(err) = self
            .repo
            .create(&mut tx, event_id, article_id, payload)
            .await
        {
            tracing::error!("Error creating calendar entry: {err}");
            let _ = tx.rollback().await;
            return Err(AppError::DatabaseError(err));
        }

        tx.commit().await.map_err(|err| {
            tracing::error!("Error committing calendar entry creation: {err}");
            AppError::InternalError
        })?;

        self.get_entry(article_id, event_id).await
    }

    async fn update_entry(
        &self,
        article_id: Uuid,
        event_id: Uuid,
        payload: UpdateCalendarEntryDto,
    ) -> Result<CalendarEntryDto, AppError> {
        validate_entry_semantics(&payload.entry_type, &payload.block_reason)?;
        validate_time_order(payload.start_time, payload.end_time)?;

        let mut client = self.pool.get().await.map_err(AppError::DatabaseError)?;
        let mut tx = client.transaction().await.map_err(|err| {
            tracing::error!("Error starting calendar entry update transaction: {err}");
            AppError::InternalError
        })?;

        match self
            .repo
            .update(&mut tx, article_id, event_id, payload)
            .await
        {
            Ok(Some(entry)) => {
                tx.commit().await.map_err(|err| {
                    tracing::error!("Error committing calendar entry update: {err}");
                    AppError::InternalError
                })?;
                Ok(CalendarEntryDto::from(entry))
            }
            Ok(None) => {
                let _ = tx.rollback().await;
                Err(AppError::NotFound("Calendar entry not found".into()))
            }
            Err(err) => {
                tracing::error!("Error updating calendar entry: {err}");
                let _ = tx.rollback().await;
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn delete_entry(&self, article_id: Uuid, event_id: Uuid) -> Result<String, AppError> {
        let mut client = self.pool.get().await.map_err(AppError::DatabaseError)?;
        let mut tx = client.transaction().await.map_err(|err| {
            tracing::error!("Error starting calendar entry deletion transaction: {err}");
            AppError::InternalError
        })?;

        match self.repo.delete(&mut tx, article_id, event_id).await {
            Ok(true) => {
                tx.commit().await.map_err(|err| {
                    tracing::error!("Error committing calendar entry deletion: {err}");
                    AppError::InternalError
                })?;
                Ok("Calendar entry deleted".into())
            }
            Ok(false) => {
                let _ = tx.rollback().await;
                Err(AppError::NotFound("Calendar entry not found".into()))
            }
            Err(err) => {
                tracing::error!("Error deleting calendar entry: {err}");
                let _ = tx.rollback().await;
                Err(AppError::DatabaseError(err))
            }
        }
    }
}
