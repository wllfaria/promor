use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::PgPool;

pub struct HandlerId(i32);

impl HandlerId {
    pub fn new_unchecked(id: i32) -> Self {
        Self(id)
    }

    pub fn inner(&self) -> i32 {
        self.0
    }
}

/// in-database store handler representation
#[derive(Debug, FromRow)]
pub struct StoreHandlerRow {
    pub id: i32,
    pub type_id: i32,
    pub api_id: Option<i32>,
    pub parser: Option<String>,
}

/// in-database store_api representation
#[derive(Debug, FromRow)]
pub struct StoreApiRow {
    pub id: i32,
    pub name: String,
}

/// in-database store_handler_type representation
#[derive(Debug, FromRow)]
pub struct StoreHandlerTypeRow {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum StoreApi {
    Amazon,
    MercadoLivre,
}

#[derive(Debug, PartialEq, Eq, Copy, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub enum StoreHandler {
    Scraper(StoreParser),
    Api(StoreApi),
}

#[derive(Debug, PartialEq, Eq, Copy, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub enum StoreParser {
    Kabum,
}

impl StoreHandler {
    pub async fn get_by_id(db: &PgPool, id: HandlerId) -> anyhow::Result<StoreHandler> {
        let handler = sqlx::query_as!(
            StoreHandlerRow,
            "SELECT * FROM store_handlers WHERE id = $1",
            id.inner()
        )
        .fetch_one(db)
        .await?;

        let handler_type = sqlx::query_as!(
            StoreHandlerTypeRow,
            "SELECT * FROM store_handler_types WHERE id = $1",
            handler.type_id
        )
        .fetch_one(db)
        .await?;

        let handler = match handler_type.name.as_str() {
            "scraper" => match handler.parser.as_deref() {
                Some("kabum_search") => StoreHandler::Scraper(StoreParser::Kabum),
                _ => anyhow::bail!("invalid store parser"),
            },
            "api" => {
                let Some(id) = handler.api_id else {
                    anyhow::bail!("handler type is api but no api_id is defined");
                };
                let store_api = sqlx::query_as!(StoreApiRow, " SELECT * FROM store_apis WHERE id = $1", id)
                    .fetch_one(db)
                    .await?;
                match store_api.name.as_str() {
                    "amazon" => StoreHandler::Api(StoreApi::Amazon),
                    "mercado_livre" => StoreHandler::Api(StoreApi::MercadoLivre),
                    api => anyhow::bail!("invalid store api {api}"),
                }
            }
            handler_type => anyhow::bail!("invalid handler_type {handler_type}"),
        };

        Ok(handler)
    }
}
