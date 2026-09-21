use bytes::Bytes;

use thiserror::Error;

pub struct HttpClient {
    lessons_change_page: &'static str,
    lessons_page: &'static str,
}

impl HttpClient {
    pub fn new(
        lessons_change_page: &'static str,
        lessons_page: &'static str,
    ) -> Self {
        Self { lessons_change_page, lessons_page }
    }

    pub async fn get_page_document(&self, page: &'static str) -> Result<Bytes, HttpError> {
        Ok(reqwest::get(page)
            .await?
            .bytes()
            .await?)
    }
}

#[derive(Error, Debug)]
pub enum HttpError {
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error)
    
}