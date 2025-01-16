pub mod kabum_product_handler;
pub mod kabum_search_handler;
pub mod page_scraper;
pub mod queue_scraper;

use std::sync::Arc;

use headless_chrome::{Browser, Tab};
use kabum_search_handler::KabumSearchHandler;
use page_scraper::PageScraper;
use queue_scraper::QueueScraper;
use sqlx::PgPool;
use tokio::sync::Semaphore;
use url::Url;

use crate::models::page::{Page, PageHandler};
use crate::models::store::StoreId;

pub trait ScrapHandler: Send {
    type Input;
    type Output;

    async fn run(&mut self, tab: Arc<Tab>, page: Self::Input) -> anyhow::Result<Self::Output>;
}

#[derive(Debug)]
pub struct QueuePage {
    pub url: Url,
    pub store_id: StoreId,
    pub handler: PageHandler,
    pub ean: Option<String>,
    pub gtin: Option<String>,
}

#[tracing::instrument(skip_all)]
pub async fn start_thread(db: PgPool) -> anyhow::Result<()> {
    tokio::spawn(async move {
        const INTERVAL_SECS: u64 = 60 * 60 * 24;
        const CONCURRENT_LIMIT: usize = 2;

        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(INTERVAL_SECS));
        let semaphore = Arc::new(Semaphore::new(CONCURRENT_LIMIT));

        loop {
            interval.tick().await;
            tracing::info!("starting scraper routine");

            let browser = match Browser::default() {
                Ok(browser) => browser,
                Err(_) => return,
            };

            let urls = match scrap_search_pages(&db, &browser, &semaphore).await {
                Ok(ScrapResult::Finished(result)) => result,
                Ok(ScrapResult::Skip) => {
                    tracing::warn!("skipping scraper routine");
                    continue;
                }
                Err(_) => continue,
            };

            match QueueScraper::new(db.clone()).run(&browser, urls).await {
                Ok(_) => tracing::info!("finished queue handler"),
                Err(_) => todo!(),
            }
        }
    });
    Ok(())
}

pub enum ScrapResult<T> {
    Finished(T),
    Skip,
}

#[tracing::instrument(skip_all)]
async fn scrap_search_pages(
    db: &PgPool,
    browser: &Browser,
    semaphore: &Arc<Semaphore>,
) -> anyhow::Result<ScrapResult<Vec<QueuePage>>> {
    tracing::info!("starting to scrap search pages");

    let Ok(pages) = Page::get_all_search_pages(db).await else {
        tracing::error!("failed to fetch stores from database");
        return Ok(ScrapResult::Skip);
    };

    let mut handles = vec![];

    for page in pages {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let browser = browser.clone();
        let db = db.clone();

        let handle = tokio::spawn(async move {
            // holding the permit until task goes out of scope to prevent over usage of
            // resources by scraping threads
            let _permit = permit;

            let store_id = page.store_id;

            let result = match page.handler {
                PageHandler::KabumSearch => {
                    PageScraper::new(
                        KabumSearchHandler::new(store_id, page.ean.clone(), page.gtin.clone()),
                        db,
                        page,
                    )
                    .run(&browser)
                    .await
                }
                PageHandler::KabumProduct => unreachable!(),
            };

            match result {
                Ok(urls) => Ok(urls),
                Err(e) => {
                    tracing::error!("failed to scrap page with error: {e}");
                    anyhow::bail!("failed to scrap page with error: {e}");
                }
            }
        });

        handles.push(handle);
    }

    let mut urls = vec![];
    for handle in handles {
        let handle_urls = handle.await.unwrap()?;
        urls.extend(handle_urls);
    }

    Ok(ScrapResult::Finished(urls))
}
