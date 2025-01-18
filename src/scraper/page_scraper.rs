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
    pub fn new(parser: P, db: PgPool, page: Page) -> Self {
        Self { db, page, parser }
    }

    pub async fn run(&mut self, browser: &Browser) -> anyhow::Result<Vec<QueuePage>> {
        let tab = browser.new_tab()?;
        let Some(store) = Store::get_by_id(&self.db, self.page.store_id).await? else {
            // TODO: if we don't have a store on a page, we have something really bad going on, so
            // this here is not the optimal error handling and should change
            return Ok(vec![]);
        };

        tab.navigate_to(self.page.url.as_str())?;

        self.parser.run(tab.clone(), store).await
    }
}
