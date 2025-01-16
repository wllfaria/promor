use axum::extract::Path;
use axum::routing::get;
use axum::{Extension, Json, Router};
use sqlx::PgPool;

use crate::error::AppError;
use crate::handlers;
use crate::models::product_price::ProductPrice;

pub fn product_price_routes() -> Router {
    Router::new()
        .route("/product_prices", get(get_all))
        .route("/product_prices/{id}", get(get_one))
}

#[axum::debug_handler]
async fn get_all(Extension(db): Extension<PgPool>) -> anyhow::Result<Json<Vec<ProductPrice>>, AppError> {
    let response = handlers::product_price::get_all(&db).await?;
    Ok(Json(response))
}

#[axum::debug_handler]
async fn get_one(
    Extension(db): Extension<PgPool>,
    Path(id): Path<i32>,
) -> anyhow::Result<Json<ProductPrice>, AppError> {
    let response = handlers::product_price::get_one(&db, id).await?;
    Ok(Json(response))
}
