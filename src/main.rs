mod error;
mod handlers;
mod models;
mod routers;
mod scrapper;

use std::sync::Arc;

use anyhow::Context;
use axum::{Extension, Router};
use models::handler::StoreHandler;
use models::store::Store;
use scrapper::Scraper;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing();

    let db = setup_db().await?;

    let app = Router::new()
        .nest("/api", Router::new().merge(routers::store::store_routes()))
        .layer(Extension(db.clone()));

    scraper_thread(db);

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

#[tracing::instrument(skip_all)]
fn scraper_thread(db: PgPool) {
    tokio::spawn(async move {
        const INTERVAL_SECS: u64 = 10;
        const CONCURRENT_LIMIT: usize = 2;

        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(INTERVAL_SECS));
        let semaphore = Arc::new(Semaphore::new(CONCURRENT_LIMIT));

        loop {
            interval.tick().await;

            let Ok(stores) = Store::get_all(&db).await else {
                tracing::error!("failed to fetch stores from database");
                continue;
            };

            for store in stores {
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let db = db.clone();

                tokio::spawn(async move {
                    // holding the permit until task goes out of scope to prevent over usage of
                    // resources by scraping threads
                    let _permit = permit;

                    let _result = match store.handler {
                        StoreHandler::Scraper(_) => Scraper::new(store, db).run().await,
                        StoreHandler::Api(_) => todo!(),
                    };
                });
            }
        }
    });
}
