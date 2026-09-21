use std::{env::var, mem::{forget, transmute}, slice::from_raw_parts};

use tracing::info;

use crate::http::HttpClient;

mod http;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().unwrap();

    let lessons_change_page = unsafe { parse_page("LESSONS_CHANGES_PAGE") };

    let http_client = HttpClient::new(lessons_change_page, "");
    
    let document = http_client.get_page_document(lessons_change_page).await;

    info!("Document: {:?}", document);
}

unsafe fn parse_page(env_name: &'static str) -> &'static str {
    let page = var(env_name).unwrap();
    let (ptr, len) = (page.as_ptr(), page.len());
    unsafe { 
        forget(page);
        transmute(from_raw_parts(ptr, len))
    }
}