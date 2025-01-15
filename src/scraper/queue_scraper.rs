use std::sync::Arc;

use headless_chrome::Browser;
use sqlx::PgPool;
use tokio::sync::Semaphore;

use super::QueuePage;
use crate::models::page::PageHandler;
use crate::scraper::kabum_product_handler::KabumProductHandler;
use crate::scraper::ScrapHandler;

pub struct QueueScraper {
    db: PgPool,
}

impl QueueScraper {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn run(&mut self, browser: &Browser, queue: Vec<QueuePage>) -> anyhow::Result<()> {
        const CONCURRENT_LIMIT: usize = 10;
        let semaphore = Arc::new(Semaphore::new(CONCURRENT_LIMIT));

        // artificial delay to prevent rate limiting
        // TODO: tweak this to a better default delay
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(300));
        let mut handles = vec![];

        for page in queue {
            interval.tick().await;

            let permit = semaphore.clone().acquire_owned().await?;
            let tab = browser.new_tab()?;

            let handle = tokio::spawn(async move {
                let _permit = permit;

                let mut handler = match page.handler {
                    PageHandler::KabumProduct => KabumProductHandler::default(),
                    PageHandler::KabumSearch => unreachable!(),
                };

                match handler.run(tab, page).await {
                    Ok(_) => {}
                    Err(e) => tracing::error!("{}", e.to_string()),
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.await?;
        }

        Ok(())
    }
}
