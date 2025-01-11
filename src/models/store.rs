use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::PgPool;
use validator::Validate;

use super::handler::{HandlerId, StoreHandler};
use crate::error::AppError;

#[derive(Debug)]
pub struct StoreId(i32);

impl StoreId {
    pub async fn new(db: &PgPool, id: i32) -> anyhow::Result<Self> {
        let result = sqlx::query!("SELECT EXISTS(SELECT 1 FROM stores WHERE id= $1)", id)
            .fetch_one(db)
            .await?;

        if result.exists.unwrap_or_default() {
            anyhow::bail!("invalid store id");
        }

        Ok(Self(id))
    }

    fn new_unchecked(id: i32) -> Self {
        Self(id)
    }
}

#[derive(Debug)]
pub struct Dirty;

#[derive(Debug)]
pub struct Valid;

#[derive(Debug, Validate, Deserialize)]
pub struct CreateStorePayload<State = Dirty> {
    #[validate(url(message = "url must be a valid url"))]
    pub url: String,
    #[validate(length(min = 1, max = 100, message = "name must have between 1 and 100 characters"))]
    pub name: String,
    #[validate()]
    pub handler_id: i32,
    #[serde(skip)]
    _marker: std::marker::PhantomData<State>,
}

impl CreateStorePayload<Dirty> {
    pub fn parse(self) -> anyhow::Result<CreateStorePayload<Valid>, AppError> {
        self.validate().map_err(AppError::ValidationError)?;

        Ok(CreateStorePayload {
            url: self.url,
            name: self.name,
            handler_id: self.handler_id,
            _marker: std::marker::PhantomData,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Store {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub handler: StoreHandler,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// in-database store representation
#[derive(Debug, FromRow)]
struct StoreRow {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub handler_id: i32,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FromSqlxRow<StoreRow> for Store {
    async fn from_sqlx(row: StoreRow, db: &PgPool) -> anyhow::Result<Self> {
        let handler = StoreHandler::get_by_id(db, HandlerId::new_unchecked(row.handler_id)).await?;

        Ok(Store {
            id: row.id,
            name: row.name,
            url: row.url,
            handler,
            active: row.active,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

pub trait FromSqlxRow<T> {
    async fn from_sqlx(row: T, db: &PgPool) -> anyhow::Result<Self>
    where
        Self: Sized;
}

impl Store {
    pub async fn get_all(db: &PgPool) -> anyhow::Result<Vec<Store>> {
        let rows = sqlx::query_as!(StoreRow, "SELECT * FROM stores").fetch_all(db).await?;
        let mut stores = vec![];

        for store in rows {
            stores.push(Store::from_sqlx(store, db).await?);
        }

        Ok(stores)
    }

    pub async fn get_by_id(db: &PgPool, id: StoreId) -> anyhow::Result<Store> {
        let row = sqlx::query_as!(StoreRow, "SELECT * FROM stores WHERE id = $1", id.0)
            .fetch_one(db)
            .await?;

        let store = Store::from_sqlx(row, db).await?;

        Ok(store)
    }

    pub async fn create(db: &PgPool, store: CreateStorePayload<Valid>) -> anyhow::Result<Store> {
        let id = sqlx::query_as!(
            Store,
            "INSERT INTO stores (url, name, handler_id) VALUES ($1, $2, $3)",
            &store.url,
            &store.name,
            &store.handler_id
        )
        .execute(db)
        .await?
        .rows_affected();

        let store = Store::get_by_id(db, StoreId::new_unchecked(id as i32)).await?;

        Ok(store)
    }
}
