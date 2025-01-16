use std::sync::Arc;

use headless_chrome::Tab;
use serde::Deserialize;
use sqlx::PgPool;
use url::Url;

use super::{QueuePage, ScrapHandler};
use crate::models::product::Product;
use crate::models::product_price::{CreateProductPricePayload, ProductPrice, ValidCreateProductPricePayload};

#[derive(Debug)]
pub struct KabumProductHandler {
    api: Url,
    db: PgPool,
}

impl KabumProductHandler {
    pub fn new(db: PgPool) -> Self {
        let url = Url::parse("https://servicespub.prod.api.aws.grupokabum.com.br/descricao/v1/descricao/produto/");
        Self { api: url.unwrap(), db }
    }
}

#[derive(Debug, Deserialize)]
pub struct KabumProductDescription {
    #[serde(rename = "codigo")]
    pub id: u64,
    #[serde(rename = "nome")]
    pub name: String,
    #[serde(rename = "disponibilidade")]
    pub availability: bool,
    #[serde(rename = "fabricante")]
    pub manufacturer: KabumManufacturer,
    #[serde(rename = "preco")]
    pub price: f64,
    #[serde(rename = "preco_antigo")]
    pub old_price: f64,
    #[serde(rename = "preco_desconto")]
    pub discount_price: f64,
}

#[derive(Debug, Deserialize)]
pub struct KabumManufacturer {
    #[serde(rename = "codigo")]
    pub id: u64,
    #[serde(rename = "nome")]
    pub name: String,
}

impl ScrapHandler for KabumProductHandler {
    type Input = QueuePage;
    type Output = ();

    async fn run(&mut self, _: Arc<Tab>, page: Self::Input) -> anyhow::Result<Self::Output> {
        // the second piece of path will always be the product ID, eg:
        // "/produto/<product_id>/..."
        let Some(product_id) = page.url.path().split("/").nth(2) else {
            anyhow::bail!("malformed product url {}", page.url.as_str());
        };

        tracing::error!("so, we got here");

        let mut url = self.api.to_string();
        url.push_str(product_id);
        let url = Url::parse(&url).unwrap();

        let response = reqwest::get(url).await?;
        let body = response.text().await?;
        let body = serde_json::from_str::<KabumProductDescription>(&body)?;

        let result = match (page.ean.as_ref(), page.gtin.as_ref()) {
            (Some(ean), _) => Product::get_by_ean(&self.db, ean).await,
            (_, Some(gtin)) => Product::get_by_gtin(&self.db, gtin).await,
            (None, None) => todo!(),
        };

        let product = match result {
            Ok(product) => product,
            Err(_) => {
                tracing::error!("product not registered on the database");
                anyhow::bail!("TODO: this should not crash the thread");
            }
        };

        let payload = CreateProductPricePayload {
            product_id: product.id.inner(),
            store_id: page.store_id.inner(),
            price: body.price,
        };

        let payload = payload.parse(&self.db).await?;

        ProductPrice::create(&self.db, payload).await?;

        Ok(())
    }
}
