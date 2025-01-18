use axum::extract::Path;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use sqlx::PgPool;

use super::HttpResponse;
use crate::error::AppError;
use crate::handlers;
use crate::models::product::{CreateProductPayload, Product};

pub fn product_routes() -> Router {
    Router::new()
        .route("/products", get(get_all))
        .route("/products", post(create))
        .route("/products/{id}", get(get_one))
}

#[axum::debug_handler]
async fn get_all(
    Extension(db): Extension<PgPool>,
) -> anyhow::Result<Json<HttpResponse<Option<Vec<Product>>>>, AppError> {
    let body = handlers::product::get_all(&db).await?;
    Ok(Json(HttpResponse::ok(body)))
}

#[axum::debug_handler]
async fn get_one(
    Extension(db): Extension<PgPool>,
    Path(id): Path<i32>,
) -> anyhow::Result<Json<HttpResponse<Option<Product>>>, AppError> {
    let body = handlers::product::get_one(&db, id).await?;
    Ok(Json(HttpResponse::ok(body)))
}

#[axum::debug_handler]
async fn create(
    Extension(db): Extension<PgPool>,
    Json(payload): Json<CreateProductPayload>,
) -> Result<Json<HttpResponse<Product>>, AppError> {
    let body = handlers::product::create(&db, payload).await?;
    Ok(Json(HttpResponse::created(body)))
}
