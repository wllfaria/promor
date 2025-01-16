use sqlx::PgPool;

use crate::error::AppError;
use crate::models::product::{CreateProductPayload, Product, ProductId};

#[tracing::instrument(skip_all)]
pub async fn get_all(db: &PgPool) -> anyhow::Result<Vec<Product>, AppError> {
    let products = Product::get_all(db).await?;
    Ok(products)
}

#[tracing::instrument(skip_all)]
pub async fn get_one(db: &PgPool, id: i32) -> anyhow::Result<Product, AppError> {
    let id = ProductId::new(db, id).await?;
    let product = Product::get_by_id(db, id).await?;
    Ok(product)
}

#[tracing::instrument(skip_all)]
pub async fn create(db: &PgPool, payload: CreateProductPayload) -> anyhow::Result<Product, AppError> {
    let payload = payload.parse()?;
    let product = Product::create(db, payload).await?;
    Ok(product)
}
