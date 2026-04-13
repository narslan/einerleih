use async_trait::async_trait;
use deadpool_postgres::{Pool, PoolError, Transaction};
use uuid::Uuid;

use crate::domains::booking::{
    domain::model::{Booking, BookingStatus},
    dto::booking_dto::{BookingFilterDto, CreateBookingDto, UpdateBookingDto},
};

#[async_trait]
pub trait BookingRepository: Send + Sync {
    async fn article_exists(&self, pool: Pool, article_id: Uuid) -> Result<bool, PoolError>;

    async fn has_confirmed_booking_conflict(
        &self,
        pool: Pool,
        article_id: Uuid,
        booking_id: Option<Uuid>,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<bool, PoolError>;

    async fn find_by_article_id(
        &self,
        pool: Pool,
        article_id: Uuid,
        filter: BookingFilterDto,
    ) -> Result<Vec<Booking>, PoolError>;

    async fn find_by_id(
        &self,
        pool: Pool,
        article_id: Uuid,
        booking_id: Uuid,
    ) -> Result<Option<Booking>, PoolError>;

    async fn create(
        &self,
        tx: &mut Transaction<'_>,
        booking_id: Uuid,
        article_id: Uuid,
        payload: CreateBookingDto,
    ) -> Result<(), PoolError>;

    async fn update(
        &self,
        tx: &mut Transaction<'_>,
        article_id: Uuid,
        booking_id: Uuid,
        payload: UpdateBookingDto,
    ) -> Result<Option<Booking>, PoolError>;

    async fn update_status(
        &self,
        tx: &mut Transaction<'_>,
        article_id: Uuid,
        booking_id: Uuid,
        status: BookingStatus,
        actor_id: Uuid,
    ) -> Result<Option<Booking>, PoolError>;
}
