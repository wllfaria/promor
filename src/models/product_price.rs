use chrono::{DateTime, Utc};
use num_traits::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::types::BigDecimal;
use sqlx::PgPool;
use validator::Validate;

use super::product::ProductId;
use super::store::StoreId;
use crate::newtype_id;

newtype_id! {
    ProductPriceId => product_prices
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductPrice {
    pub id: ProductPriceId,
    #[serde(rename = "productId")]
    pub product_id: ProductId,
    #[serde(rename = "storeId")]
    pub store_id: StoreId,
    pub price: f64,
    pub active: bool,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "deletedAt")]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, FromRow)]
pub struct ProductPriceRow {
    pub id: i32,
    pub product_id: i32,
    pub store_id: i32,
    pub price: BigDecimal,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl From<ProductPriceRow> for ProductPrice {
    fn from(product_price: ProductPriceRow) -> Self {
        Self {
            id: ProductPriceId::new_unchecked(product_price.id),
            product_id: ProductId::new_unchecked(product_price.product_id),
            store_id: StoreId::new_unchecked(product_price.store_id),
            price: product_price.price.to_f64().unwrap_or_default(),
            active: product_price.active,
            created_at: product_price.created_at,
            updated_at: product_price.updated_at,
            deleted_at: product_price.deleted_at,
        }
    }
}

#[derive(Debug, Validate)]
pub struct CreateProductPricePayload {
    pub product_id: i32,
    pub store_id: i32,
    #[validate(range(min = 0.0, max = f64::MAX, message = "price cannot be negative"))]
    pub price: f64,
}

pub struct ValidCreateProductPricePayload {
    product_id: ProductId,
    store_id: StoreId,
    price: BigDecimal,
}

impl CreateProductPricePayload {
    pub async fn parse(self, db: &PgPool) -> anyhow::Result<ValidCreateProductPricePayload> {
        let product_id = ProductId::new(db, self.product_id).await?;
        let store_id = StoreId::new(db, self.store_id).await?;
        let price = BigDecimal::from_f64(self.price).unwrap();

        Ok(ValidCreateProductPricePayload {
            product_id,
            store_id,
            price,
        })
    }
}

impl ProductPrice {
    pub async fn get_all(db: &PgPool) -> anyhow::Result<Option<Vec<ProductPrice>>> {
        let result = sqlx::query_as!(ProductPriceRow, "SELECT * FROM product_prices")
            .fetch_all(db)
            .await;

        match result {
            Ok(prices) => Ok(Some(prices.into_iter().map(Into::into).collect())),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn get_by_id(db: &PgPool, id: ProductPriceId) -> anyhow::Result<Option<ProductPrice>> {
        let store = sqlx::query_as!(
            ProductPriceRow,
            "SELECT * FROM product_prices WHERE id = $1",
            id.inner()
        )
        .fetch_optional(db)
        .await?
        .map(Into::into);

        Ok(store)
    }

    pub async fn create(db: &PgPool, payload: ValidCreateProductPricePayload) -> anyhow::Result<ProductPrice> {
        let product = sqlx::query_as!(
            ProductPriceRow,
            r#"
            INSERT INTO product_prices (product_id, store_id, price)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            &payload.product_id.inner(),
            &payload.store_id.inner(),
            &payload.price,
        )
        .fetch_one(db)
        .await?
        .into();

        Ok(product)
    }
}
