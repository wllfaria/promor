use std::sync::Arc;

use headless_chrome::Tab;
use url::Url;

use super::{PageParser, QueuePage};
use crate::models::store::Store;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KabumSearchParser;

impl PageParser for KabumSearchParser {
    fn parse(&mut self, tab: Arc<Tab>, store: Store) -> anyhow::Result<Vec<QueuePage>> {
        tab.wait_for_element(".productCard")?;
        let products = tab.find_elements(".productCard")?;
        let mut links = vec![];

        for product in products {
            let Some(url) = product.find_element(".productLink")?.get_attribute_value("href")? else {
                continue;
            };

            // product URLs on search pages contains only the suffix after the main domain.
            // we prepend the base url for the store here
            let mut full_url = store.url.to_string();
            full_url.push_str(&url);

            let url = Url::parse(&full_url)?;
            links.push(url)
        }

        Ok(vec![])
    }
}
