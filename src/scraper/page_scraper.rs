use headless_chrome::Browser;
use sqlx::PgPool;

use super::{QueuePage, ScrapHandler};
use crate::models::page::Page;
use crate::models::store::Store;

pub struct PageScraper<P>
where
    P: ScrapHandler<Input = Store, Output = Vec<QueuePage>>,
{
    db: PgPool,
    page: Page,
    parser: P,
}

impl<P> PageScraper<P>
where
    P: ScrapHandler<Input = Store, Output = Vec<QueuePage>>,
{
    pub fn new(page: Page, db: PgPool, parser: P) -> Self {
        Self { db, page, parser }
    }

    pub async fn run(&mut self, browser: &Browser) -> anyhow::Result<Vec<QueuePage>> {
        let tab = browser.new_tab()?;
        let store = Store::get_by_id(&self.db, self.page.store_id).await?;

        tab.navigate_to(self.page.url.as_str())?;

        self.parser.run(tab.clone(), store).await
    }
}
