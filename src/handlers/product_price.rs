use sqlx::PgPool;

use crate::error::AppError;
use crate::models::product_price::{ProductPrice, ProductPriceId};

#[tracing::instrument(skip_all)]
pub async fn get_all(db: &PgPool) -> anyhow::Result<Vec<ProductPrice>> {
    let product_prices = ProductPrice::get_all(db).await?;
    Ok(product_prices)
}

#[tracing::instrument(skip_all)]
pub async fn get_one(db: &PgPool, id: i32) -> anyhow::Result<ProductPrice, AppError> {
    let id = ProductPriceId::new(db, id).await?;
    let product_price = ProductPrice::get_by_id(db, id).await?;
    Ok(product_price)
}
