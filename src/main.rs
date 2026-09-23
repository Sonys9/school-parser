use std::{
    env::var,
    mem::{forget, transmute},
    slice::from_raw_parts,
};

use dhat::{Alloc, Profiler};
use tokio::time::Instant;
use tracing::info;

use crate::{http::HttpClient, parser::Parser};

mod errors;
mod http;
mod parser;

#[global_allocator]
static ALLOC: Alloc = Alloc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().unwrap();

    let lessons_change_page = unsafe { parse_page("LESSONS_CHANGES_PAGE") };
    let http_client = HttpClient::new(lessons_change_page, "");

    let document = unsafe { http_client.get_page_document(lessons_change_page).await }.unwrap();

    let profiler = Profiler::new_heap();
    let time = Instant::now();

    let parser = Parser::new(&document);
    let changes = parser.changes();

    info!(
        "Parsed changes: {:?} in {}ns",
        changes,
        time.elapsed().as_nanos()
    );

    drop(profiler);
}

unsafe fn parse_page(env_name: &'static str) -> &'static str {
    let page = var(env_name).unwrap();
    let (ptr, len) = (page.as_ptr(), page.len());
    unsafe {
        forget(page);
        transmute(from_raw_parts(ptr, len))
    }
}
