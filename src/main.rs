use std::{collections::HashMap, env::var, mem::{forget, transmute}, slice::from_raw_parts};

use tl::{Parser, ParserOptions};
use tokio::time::Instant;
use tracing::info;
use std::borrow::Cow;

use crate::{http::{HttpClient, Str}, parser::parse_changes};

mod http;
mod errors;
mod parser;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().unwrap();

    let lessons_change_page = unsafe { parse_page("LESSONS_CHANGES_PAGE") };
    let http_client = HttpClient::new(lessons_change_page, "");
    
    let document = unsafe { http_client.get_page_document(lessons_change_page).await }.unwrap();

    let time = Instant::now();
    let changes = unsafe { parse_changes(&document) };

    info!("Parsed changes: {:?} in {}ns", changes, time.elapsed().as_nanos());
}

unsafe fn parse_page(env_name: &'static str) -> &'static str {
    let page = var(env_name).unwrap();
    let (ptr, len) = (page.as_ptr(), page.len());
    unsafe { 
        forget(page);
        transmute(from_raw_parts(ptr, len))
    }
}