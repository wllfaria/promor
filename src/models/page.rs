use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::PgPool;
use url::Url;
use validator::Validate;

use super::store::StoreId;
use crate::newtype_id;

newtype_id! {
    PageId => pages
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize)]
pub enum PageHandler {
    KabumSearch,
    KabumProduct,
}

impl PageHandler {
    pub fn inner(&self) -> &str {
        match self {
            PageHandler::KabumSearch => "kabum_search",
            PageHandler::KabumProduct => "kabum_product",
        }
    }
}

impl TryFrom<String> for PageHandler {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_ref() {
            "kabum_search" => Ok(Self::KabumSearch),
            _ => anyhow::bail!("invalid page handler"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize)]
pub enum PageKind {
    Search,
    Details,
}

impl PageKind {
    pub fn inner(&self) -> &str {
        match self {
            PageKind::Search => "search",
            PageKind::Details => "details",
        }
    }
}

impl TryFrom<String> for PageKind {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_ref() {
            "search" => Ok(Self::Search),
            "details" => Ok(Self::Details),
            _ => anyhow::bail!("invalid page kind"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    pub id: PageId,
    pub name: String,
    pub url: Url,
    #[serde(rename = "storeId")]
    pub store_id: StoreId,
    pub handler: PageHandler,
    #[serde(rename = "pageKind")]
    pub page_kind: PageKind,
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
struct PageRow {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub store_id: i32,
    pub handler: String,
    pub page_kind: String,
    pub ean: Option<String>,
    pub gtin: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePagePayload {
    #[validate(length(min = 1, max = 100, message = "name of page must have between 1 and 100 characters"))]
    pub name: String,
    #[validate(url(message = "page url must be valid"))]
    pub url: String,
    #[serde(rename = "storeId")]
    pub store_id: i32,
    pub handler: String,
    #[serde(rename = "pageKind")]
    pub page_kind: String,
}

#[derive(Debug, Serialize)]
pub struct ValidCreatePagePayload {
    pub name: String,
    pub url: Url,
    pub store_id: StoreId,
    pub handler: PageHandler,
    pub page_kind: PageKind,
}

impl CreatePagePayload {
    pub async fn parse(self, db: &PgPool) -> anyhow::Result<ValidCreatePagePayload> {
        self.validate()?;

        let url = Url::parse(&self.url)?;
        let store_id = StoreId::new(db, self.store_id).await?;
        let handler = self.handler.try_into()?;
        let page_kind = self.page_kind.try_into()?;

        Ok(ValidCreatePagePayload {
            name: self.name,
            url,
            store_id,
            handler,
            page_kind,
        })
    }
}

impl From<PageRow> for Page {
    fn from(value: PageRow) -> Self {
        Self {
            id: PageId::new_unchecked(value.id),
            name: value.name,
            url: Url::parse(&value.url).expect("url should be valid when querying the database"),
            store_id: StoreId::new_unchecked(value.store_id),
            handler: PageHandler::try_from(value.handler).expect("invalid page handler on the database"),
            page_kind: PageKind::try_from(value.page_kind).expect("invalid page kind on the database"),
            active: value.active,
            ean: value.ean,
            gtin: value.gtin,
            created_at: value.created_at,
            updated_at: value.updated_at,
            deleted_at: value.deleted_at,
        }
    }
}

impl Page {
    pub async fn get_all(db: &PgPool) -> anyhow::Result<Option<Vec<Page>>> {
        let result = sqlx::query_as!(PageRow, "SELECT * FROM pages WHERE active = true")
            .fetch_all(db)
            .await;

        match result {
            Ok(pages) => Ok(Some(pages.into_iter().map(Into::into).collect())),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(other) => Err(other.into()),
        }
    }

    pub async fn get_all_search_pages(db: &PgPool) -> anyhow::Result<Option<Vec<Page>>> {
        let result = sqlx::query_as!(
            PageRow,
            "SELECT * FROM pages WHERE page_kind = 'search' AND active = true"
        )
        .fetch_all(db)
        .await;

        match result {
            Ok(pages) => Ok(Some(pages.into_iter().map(Into::into).collect())),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(other) => Err(other.into()),
        }
    }

    pub async fn get_by_id(db: &PgPool, id: PageId) -> anyhow::Result<Option<Page>> {
        let page = sqlx::query_as!(
            PageRow,
            "SELECT * FROM pages WHERE id = $1 AND active = true",
            id.inner()
        )
        .fetch_optional(db)
        .await?
        .map(Into::into);

        Ok(page)
    }

    pub async fn create(db: &PgPool, page: ValidCreatePagePayload) -> anyhow::Result<Page> {
        let page = sqlx::query_as!(
            PageRow,
            r#"
            INSERT INTO pages (name, url, store_id, handler, page_kind)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            &page.name,
            page.url.as_str(),
            page.store_id.inner(),
            page.handler.inner(),
            page.page_kind.inner(),
        )
        .fetch_one(db)
        .await?
        .into();

        Ok(page)
    }
}
