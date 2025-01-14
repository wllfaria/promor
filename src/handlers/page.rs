use sqlx::PgPool;

use crate::error::AppError;
use crate::models::page::{CreatePagePayload, Page, PageId};

#[tracing::instrument(skip_all)]
pub async fn get_all(db: &PgPool) -> anyhow::Result<Vec<Page>> {
    let stores = Page::get_all(db).await?;
    Ok(stores)
}

#[tracing::instrument(skip_all)]
pub async fn get_one(db: &PgPool, id: i32) -> anyhow::Result<Page, AppError> {
    let id = PageId::new(db, id).await?;
    let store = Page::get_by_id(db, id).await?;
    Ok(store)
}

#[tracing::instrument(skip_all)]
pub async fn create(db: &PgPool, payload: CreatePagePayload) -> anyhow::Result<Page, AppError> {
    let payload = payload.parse(db).await?;
    let store = Page::create(db, payload).await?;
    Ok(store)
}
