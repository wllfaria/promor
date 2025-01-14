pub mod kabum_search_parser;
pub mod page_scraper;
pub mod queue_scraper;

use std::sync::Arc;

use headless_chrome::{Browser, Tab};
use kabum_search_parser::KabumSearchParser;
use page_scraper::PageScraper;
use queue_scraper::QueueScraper;
use sqlx::PgPool;
use tokio::sync::Semaphore;
use url::Url;

use crate::models::page::{Page, PageHandler};
use crate::models::store::{Store, StoreId};

pub trait PageParser: Send {
    fn parse(&mut self, tab: Arc<Tab>, store: Store) -> anyhow::Result<Vec<QueuePage>>;
}

#[derive(Debug)]
pub struct QueuePage {
    pub url: Url,
    pub store_id: StoreId,
    pub handler: PageHandler,
}

#[tracing::instrument(skip_all)]
pub fn start_thread(db: PgPool) {
    tokio::spawn(async move {
        const INTERVAL_SECS: u64 = 60 * 60 * 24;
        const CONCURRENT_LIMIT: usize = 2;

        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(INTERVAL_SECS));
        let semaphore = Arc::new(Semaphore::new(CONCURRENT_LIMIT));

        loop {
            interval.tick().await;

            let urls = match scrap_search_pages(&db, &semaphore).await {
                Ok(ScrapResult::Finished(result)) => result,
                Ok(ScrapResult::Skip) => continue,
                Err(_) => continue,
            };

            match QueueScraper::new(db.clone(), urls).run().await {
                Ok(_) => todo!(),
                Err(_) => todo!(),
            }
        }
    });
}

pub enum ScrapResult<T> {
    Finished(T),
    Skip,
}

async fn scrap_search_pages(db: &PgPool, semaphore: &Arc<Semaphore>) -> anyhow::Result<ScrapResult<Vec<QueuePage>>> {
    let Ok(pages) = Page::get_all_search_pages(db).await else {
        tracing::error!("failed to fetch stores from database");
        return Ok(ScrapResult::Skip);
    };

    let mut handles = vec![];
    let browser = Browser::default()?;

    for page in pages {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let browser = browser.clone();
        let db = db.clone();

        let handle = tokio::spawn(async move {
            // holding the permit until task goes out of scope to prevent over usage of
            // resources by scraping threads
            let _permit = permit;

            let result = match page.handler {
                PageHandler::KabumSearch => PageScraper::new(page, db, KabumSearchParser).run(&browser).await,
            };

            match result {
                Ok(urls) => urls,
                Err(e) => {
                    tracing::error!("failed to scrap page with error: {e}");
                    vec![]
                }
            }
        });

        handles.push(handle);
    }

    let mut urls = vec![];
    for handle in handles {
        let handle_urls = handle.await.unwrap();
        urls.extend(handle_urls);
    }

    Ok(ScrapResult::Finished(urls))
}
