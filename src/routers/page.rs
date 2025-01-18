use axum::extract::Path;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use sqlx::PgPool;

use super::HttpResponse;
use crate::error::AppError;
use crate::handlers;
use crate::models::page::{CreatePagePayload, Page};

pub fn page_routes() -> Router {
    Router::new()
        .route("/pages", get(get_all))
        .route("/pages", post(create))
        .route("/pages/{id}", get(get_one))
}

#[axum::debug_handler]
async fn get_all(Extension(db): Extension<PgPool>) -> anyhow::Result<Json<HttpResponse<Option<Vec<Page>>>>, AppError> {
    let response = handlers::page::get_all(&db).await?;
    Ok(Json(HttpResponse::ok(response)))
}

#[axum::debug_handler]
async fn get_one(
    Extension(db): Extension<PgPool>,
    Path(id): Path<i32>,
) -> anyhow::Result<Json<HttpResponse<Option<Page>>>, AppError> {
    let response = handlers::page::get_one(&db, id).await?;
    Ok(Json(HttpResponse::ok(response)))
}

#[axum::debug_handler]
async fn create(
    Extension(db): Extension<PgPool>,
    Json(payload): Json<CreatePagePayload>,
) -> Result<Json<HttpResponse<Page>>, AppError> {
    let response = handlers::page::create(&db, payload).await?;
    Ok(Json(HttpResponse::created(response)))
}
