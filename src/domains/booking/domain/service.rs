use std::sync::Arc;

use async_trait::async_trait;
use deadpool_postgres::Pool;
use uuid::Uuid;

use crate::{
    common::error::AppError,
    domains::booking::dto::booking_dto::{
        BookingDto, BookingFilterDto, CreateBookingDto, UpdateBookingDto,
    },
};

#[async_trait]
pub trait BookingServiceTrait: Send + Sync {
    fn create_service(pool: Pool) -> Arc<dyn BookingServiceTrait>
    where
        Self: Sized;

    async fn get_bookings_for_article(
        &self,
        article_id: Uuid,
        filter: BookingFilterDto,
    ) -> Result<Vec<BookingDto>, AppError>;

    async fn get_booking(&self, article_id: Uuid, booking_id: Uuid)
    -> Result<BookingDto, AppError>;

    async fn create_booking(
        &self,
        article_id: Uuid,
        payload: CreateBookingDto,
    ) -> Result<BookingDto, AppError>;

    async fn update_booking(
        &self,
        article_id: Uuid,
        booking_id: Uuid,
        payload: UpdateBookingDto,
    ) -> Result<BookingDto, AppError>;

    async fn confirm_booking(
        &self,
        article_id: Uuid,
        booking_id: Uuid,
        actor_id: Uuid,
    ) -> Result<BookingDto, AppError>;

    async fn reject_booking(
        &self,
        article_id: Uuid,
        booking_id: Uuid,
        actor_id: Uuid,
    ) -> Result<BookingDto, AppError>;

    async fn cancel_booking(
        &self,
        article_id: Uuid,
        booking_id: Uuid,
        actor_id: Uuid,
    ) -> Result<BookingDto, AppError>;

    async fn complete_booking(
        &self,
        article_id: Uuid,
        booking_id: Uuid,
        actor_id: Uuid,
    ) -> Result<BookingDto, AppError>;
}
