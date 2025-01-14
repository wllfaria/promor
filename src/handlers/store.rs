use sqlx::PgPool;

use crate::error::AppError;
use crate::models::store::{CreateStorePayload, Store, StoreId};

#[tracing::instrument(skip_all)]
pub async fn get_all(db: &PgPool) -> anyhow::Result<Vec<Store>> {
    let stores = Store::get_all(db).await?;
    Ok(stores)
}

#[tracing::instrument(skip_all)]
pub async fn get_one(db: &PgPool, id: i32) -> anyhow::Result<Store, AppError> {
    let id = StoreId::new(db, id).await?;
    let store = Store::get_by_id(db, id).await?;
    Ok(store)
}

#[tracing::instrument(skip_all)]
pub async fn create(db: &PgPool, payload: CreateStorePayload) -> anyhow::Result<Store, AppError> {
    let payload = payload.parse()?;
    let store = Store::create(db, payload).await?;
    Ok(store)
}
