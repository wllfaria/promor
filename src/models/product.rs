use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::PgPool;
use url::Url;
use validator::Validate;

use crate::newtype_id;

newtype_id! {
    ProductId => products
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    pub id: ProductId,
    pub name: String,
    pub brand: String,
    pub url: Option<String>,
    pub image: Option<String>,
    pub ean: Option<String>,
    pub gtin: Option<String>,
    pub active: bool,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "deletedAt")]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, FromRow)]
pub struct ProductRow {
    id: i32,
    name: String,
    url: Option<String>,
    brand: String,
    image: Option<String>,
    ean: Option<String>,
    gtin: Option<String>,
    active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl From<ProductRow> for Product {
    fn from(product: ProductRow) -> Self {
        Self {
            id: ProductId::new_unchecked(product.id),
            name: product.name,
            url: product.url,
            brand: product.brand,
            image: product.image,
            ean: product.ean,
            gtin: product.gtin,
            active: product.active,
            created_at: product.created_at,
            updated_at: product.updated_at,
            deleted_at: product.deleted_at,
        }
    }
}

#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct CreateProductPayload {
    #[validate(length(min = 1, message = "name must not be empty"))]
    pub name: String,
    #[validate(length(min = 1, message = "brand must not be empty"))]
    pub brand: String,
    #[validate(url(message = "url cannot be malformed"))]
    pub url: Option<String>,
    pub image: Option<String>,
    pub ean: Option<String>,
    pub gtin: Option<String>,
}

pub struct ValidCreateProductPayload {
    pub name: String,
    pub brand: String,
    pub url: Option<Url>,
    pub image: Option<String>,
    pub ean: Option<String>,
    pub gtin: Option<String>,
}

impl CreateProductPayload {
    pub fn parse(self) -> anyhow::Result<ValidCreateProductPayload> {
        self.validate()?;

        Ok(ValidCreateProductPayload {
            name: self.name,
            // Safety: we validated the url above
            url: self.url.map(|url| Url::parse(&url).unwrap()),
            brand: self.brand,
            image: self.image,
            ean: self.ean,
            gtin: self.gtin,
        })
    }
}

impl Product {
    pub async fn get_all(db: &PgPool) -> anyhow::Result<Option<Vec<Product>>> {
        let result = sqlx::query_as!(ProductRow, "SELECT * FROM products WHERE active = true")
            .fetch_all(db)
            .await;

        match result {
            Ok(products) => Ok(Some(products.into_iter().map(Into::into).collect())),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(other) => Err(other.into()),
        }
    }

    pub async fn get_by_id(db: &PgPool, id: ProductId) -> anyhow::Result<Option<Product>> {
        let product = sqlx::query_as!(
            ProductRow,
            "SELECT * FROM products WHERE active = true AND id = $1",
            id.inner()
        )
        .fetch_optional(db)
        .await?
        .map(Into::into);

        Ok(product)
    }

    pub async fn get_by_ean(db: &PgPool, ean: &str) -> anyhow::Result<Option<Product>> {
        let product = sqlx::query_as!(
            ProductRow,
            "SELECT * FROM products WHERE active = true AND ean = $1",
            ean
        )
        .fetch_optional(db)
        .await?
        .map(Into::into);

        Ok(product)
    }

    pub async fn get_by_gtin(db: &PgPool, gtin: &str) -> anyhow::Result<Option<Product>> {
        let product = sqlx::query_as!(
            ProductRow,
            "SELECT * FROM products WHERE active = true AND gtin = $1",
            gtin
        )
        .fetch_optional(db)
        .await?
        .map(Into::into);

        Ok(product)
    }

    pub async fn get_by_url(db: &PgPool, url: &Url) -> anyhow::Result<Option<Product>> {
        let product = sqlx::query_as!(
            ProductRow,
            "SELECT * FROM products WHERE active = true AND url = $1",
            url.as_str()
        )
        .fetch_optional(db)
        .await?
        .map(Into::into);

        Ok(product)
    }

    pub async fn create(db: &PgPool, product: ValidCreateProductPayload) -> anyhow::Result<Product> {
        let product = sqlx::query_as!(
            ProductRow,
            r#"
            INSERT INTO products (name, brand, url, image, ean, gtin)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            &product.name,
            &product.brand,
            product.url.as_ref().map(|url| url.as_str()),
            product.image.as_ref(),
            product.ean.as_ref(),
            product.gtin.as_ref(),
        )
        .fetch_one(db)
        .await?
        .into();

        Ok(product)
    }
}
