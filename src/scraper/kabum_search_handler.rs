use std::sync::Arc;

use headless_chrome::Tab;

use super::{QueuePage, ScrapHandler};
use crate::models::page::PageHandler;
use crate::models::store::{Store, StoreId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KabumSearchHandler {
    store_id: StoreId,
}

impl KabumSearchHandler {
    pub fn new(store_id: StoreId) -> Self {
        Self { store_id }
    }
}

impl ScrapHandler for KabumSearchHandler {
    type Input = Store;
    type Output = Vec<QueuePage>;

    async fn run(&mut self, tab: Arc<Tab>, store: Self::Input) -> anyhow::Result<Self::Output> {
        tab.wait_for_element(".productCard")?;
        let products = tab.find_elements(".productCard")?;
        let mut links = vec![];

        for product in products {
            let Some(url) = product.find_element(".productLink")?.get_attribute_value("href")? else {
                continue;
            };

            // product URLs on search pages contains only the suffix after the main domain.
            // we prepend the base url for the store here
            let mut full_url = store.url.clone();
            full_url.set_path(&url);

            links.push(QueuePage {
                url: full_url,
                store_id: self.store_id,
                handler: PageHandler::KabumProduct,
            })
        }

        Ok(links)
    }
}
