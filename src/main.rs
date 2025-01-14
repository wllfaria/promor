mod error;
mod handlers;
mod models;
mod routers;
mod scraper;

use anyhow::Context;
use axum::{Extension, Router};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing();

    let db = setup_db().await?;

    let api_routes = Router::new()
        .merge(routers::store::store_routes())
        .merge(routers::page::page_routes());

    let app = Router::new().nest("/api", api_routes).layer(Extension(db.clone()));

    scraper::start_thread(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3333").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn setup_db() -> anyhow::Result<PgPool> {
    let db_url = dotenvy::var("DATABASE_URL").context("DATABASE_URL env var must be set")?;
    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&db_url)
        .await
        .context("failed to connect to DATABASE_URL")?;

    sqlx::migrate!().run(&db).await.context("failed to run migrations")?;

    Ok(db)
}

fn setup_tracing() {
    tracing_subscriber::fmt()
        .pretty()
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::INFO)
        .init();
}
