use axum::extract::Path;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use sqlx::PgPool;

use crate::error::AppError;
use crate::handlers;
use crate::models::store::{CreateStorePayload, Dirty, Store};

pub fn store_routes() -> Router {
    Router::new()
        .route("/stores", get(get_all))
        .route("/stores", post(create))
        .route("/stores/{id}", get(get_one))
}

#[axum::debug_handler]
async fn get_all(Extension(db): Extension<PgPool>) -> anyhow::Result<Json<Vec<Store>>, AppError> {
    let response = handlers::store::get_all(&db).await?;
    Ok(Json(response))
}

#[axum::debug_handler]
async fn get_one(Extension(db): Extension<PgPool>, Path(id): Path<i32>) -> anyhow::Result<Json<Store>, AppError> {
    let response = handlers::store::get_one(&db, id).await?;
    Ok(Json(response))
}

#[axum::debug_handler]
async fn create(
    Extension(db): Extension<PgPool>,
    Json(payload): Json<CreateStorePayload<Dirty>>,
) -> Result<Json<Store>, AppError> {
    let response = handlers::store::create(&db, payload).await?;
    Ok(Json(response))
}
