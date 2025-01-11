use std::sync::Arc;

use headless_chrome::{Browser, Tab};
use sqlx::PgPool;

use crate::models::handler::{StoreHandler, StoreParser};
use crate::models::store::Store;

pub trait PageParser: Send {
    fn parse(&mut self, db: PgPool, tab: Arc<Tab>) -> anyhow::Result<Vec<String>>;
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KabumParser;

impl PageParser for KabumParser {
    fn parse(&mut self, db: PgPool, tab: Arc<Tab>) -> anyhow::Result<Vec<String>> {
        tab.wait_for_element(".productCard")?;
        let products = tab.find_elements(".productCard")?;

        for product in products {
            let name = product.find_element(".nameCard")?.get_inner_text()?;
            let price = product.find_element(".priceCard")?.get_inner_text()?;
            let image = product.find_element(".imageCard")?.get_attribute_value("src")?.unwrap();
            tracing::error!("{:?}", (name, price, image));
        }

        Ok(vec![])
    }
}

pub struct Scraper {
    db: PgPool,
    store: Store,
    parser: Box<dyn PageParser>,
}

impl Scraper {
    pub fn new(store: Store, db: PgPool) -> Self {
        let StoreHandler::Scraper(parser) = store.handler else {
            unreachable!("store has handler scraper but no parser");
        };

        let parser = match parser {
            StoreParser::Kabum => Box::new(KabumParser),
        };

        Self { db, store, parser }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let browser = Browser::default()?;
        let tab = browser.new_tab()?;

        tab.navigate_to(&self.store.url)?;

        let result = self.parser.parse(self.db.clone(), tab.clone())?;

        Ok(())
    }
}
