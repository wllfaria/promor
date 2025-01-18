pub mod page;
pub mod product;
pub mod product_price;
pub mod store;

use reqwest::StatusCode;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HttpResponse<T> {
    status: u16,
    ok: bool,
    body: T,
}

impl<T> HttpResponse<T> {
    pub fn new(ok: bool, body: T, status: StatusCode) -> Self {
        Self {
            ok,
            body,
            status: status.as_u16(),
        }
    }

    pub fn ok(body: T) -> Self {
        Self::new(true, body, StatusCode::OK)
    }

    pub fn created(body: T) -> Self {
        Self::new(true, body, StatusCode::CREATED)
    }
}
