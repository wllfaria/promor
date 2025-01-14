use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::Semaphore;

use super::QueuePage;

pub struct QueueScraper {
    db: PgPool,
    queue: Vec<QueuePage>,
}

impl QueueScraper {
    pub fn new(db: PgPool, queue: Vec<QueuePage>) -> Self {
        Self { db, queue }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        const CONCURRENT_LIMIT: usize = 10;

        let semaphore = Arc::new(Semaphore::new(CONCURRENT_LIMIT));

        for page in self.queue.iter_mut() {
            let permit = semaphore.clone().acquire_owned().await?;

            tokio::spawn(async move {
                let _permit = permit;

                // TODO: handle internal urls
            });
        }

        Ok(())
    }
}
