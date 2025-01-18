use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::PgPool;
use url::Url;
use validator::Validate;

use crate::error::AppError;
use crate::newtype_id;

newtype_id! {
    StoreId => stores
}

#[derive(Debug, Serialize)]
pub struct Store {
    pub id: StoreId,
    pub name: String,
    pub url: Url,
    pub active: bool,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "deletedAt")]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, FromRow)]
pub struct StoreRow {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl From<StoreRow> for Store {
    fn from(value: StoreRow) -> Self {
        Store {
            id: StoreId::new_unchecked(value.id),
            name: value.name,
            url: Url::parse(&value.url).expect("url should be valid when querying the database"),
            active: value.active,
            created_at: value.created_at,
            updated_at: value.updated_at,
            deleted_at: value.deleted_at,
        }
    }
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateStorePayload {
    #[validate(url(message = "url must be a valid url"))]
    pub url: String,
    #[validate(length(min = 1, max = 100, message = "name must have between 1 and 100 characters"))]
    pub name: String,
}

#[derive(Debug)]
pub struct ValidCreateStorePayload {
    pub name: String,
    pub url: Url,
}

impl CreateStorePayload {
    pub fn parse(self) -> anyhow::Result<ValidCreateStorePayload, AppError> {
        self.validate().map_err(AppError::ValidationError)?;

        Ok(ValidCreateStorePayload {
            url: Url::parse(&self.url).unwrap(),
            name: self.name,
        })
    }
}

impl Store {
    pub async fn get_all(db: &PgPool) -> anyhow::Result<Option<Vec<Store>>> {
        let result = sqlx::query_as!(StoreRow, "SELECT * FROM stores WHERE active = true")
            .fetch_all(db)
            .await;

        match result {
            Ok(stores) => Ok(Some(stores.into_iter().map(Into::into).collect())),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn get_by_id(db: &PgPool, id: StoreId) -> anyhow::Result<Option<Store>> {
        let store = sqlx::query_as!(StoreRow, "SELECT * FROM stores WHERE id = $1", id.inner())
            .fetch_optional(db)
            .await?
            .map(Into::into);

        Ok(store)
    }

    pub async fn create(db: &PgPool, page: ValidCreateStorePayload) -> anyhow::Result<Store> {
        let store = sqlx::query_as!(
            StoreRow,
            r#"
            INSERT INTO stores (url, name)
            VALUES ($1, $2)
            RETURNING *
            "#,
            &page.url.to_string(),
            &page.name,
        )
        .fetch_one(db)
        .await?
        .into();

        Ok(store)
    }
}
