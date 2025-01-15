use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::PgPool;
use url::Url;
use validator::Validate;

use super::store::StoreId;

#[derive(Debug, Serialize, Deserialize)]
pub struct PageId(i32);

impl PageId {
    pub async fn new(db: &PgPool, id: i32) -> anyhow::Result<Self> {
        let result = sqlx::query!("SELECT EXISTS(SELECT 1 FROM pages WHERE id = $1)", id)
            .fetch_one(db)
            .await?;

        if !result.exists.unwrap_or_default() {
            anyhow::bail!("invalid page id");
        }

        Ok(Self(id))
    }

    pub fn new_unchecked(id: i32) -> Self {
        Self(id)
    }

    pub fn inner(&self) -> i32 {
        self.0
    }
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
    pub id: i32,
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
            id: value.id,
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
    pub async fn get_all(db: &PgPool) -> anyhow::Result<Vec<Page>> {
        let pages = sqlx::query_as!(PageRow, "SELECT * FROM pages")
            .fetch_all(db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(pages)
    }

    pub async fn get_all_search_pages(db: &PgPool) -> anyhow::Result<Vec<Page>> {
        let pages = sqlx::query_as!(PageRow, "SELECT * FROM pages WHERE page_kind = 'search'")
            .fetch_all(db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(pages)
    }

    pub async fn get_by_id(db: &PgPool, id: PageId) -> anyhow::Result<Page> {
        let page = sqlx::query_as!(PageRow, "SELECT * FROM pages WHERE id = $1", id.inner())
            .fetch_one(db)
            .await?
            .into();

        Ok(page)
    }

    pub async fn create(db: &PgPool, page: ValidCreatePagePayload) -> anyhow::Result<Page> {
        let id = sqlx::query_as!(
            PageRow,
            "INSERT INTO pages (name, url, store_id, handler, page_kind) VALUES ($1, $2, $3, $4, $5)",
            &page.name,
            page.url.as_str(),
            page.store_id.inner(),
            page.handler.inner(),
            page.page_kind.inner(),
        )
        .execute(db)
        .await?
        .rows_affected();

        let page = Page::get_by_id(db, PageId::new_unchecked(id as i32)).await?;

        Ok(page)
    }
}
