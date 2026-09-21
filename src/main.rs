use std::{env::var, mem::{forget, transmute}, slice::from_raw_parts};

use tl::{Parser, ParserOptions};
use tracing::info;

use crate::http::{HttpClient, Str};

mod http;
mod errors;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().unwrap();

    let lessons_change_page = unsafe { parse_page("LESSONS_CHANGES_PAGE") };
    let http_client = HttpClient::new(lessons_change_page, "");
    
    let document = unsafe { http_client.get_page_document(lessons_change_page).await }.unwrap();
    let dom = tl::parse(&document, ParserOptions::default()).unwrap();
    
    for node in dom.nodes() {
        let Some(element) = node.as_tag() else {
            continue;
        };
        let attributes = element.attributes();
        let (Some(Some(class)), Some(Some(style))) = (attributes.get("class"), attributes.get("style")) else {
            continue
        };
        if style.as_utf8_str().contains("width:") || !(class == "xl68" || class == "xl70") {
            continue;
        };
        info!("{:?}", element.inner_text(dom.parser()));
    }
}

unsafe fn parse_page(env_name: &'static str) -> &'static str {
    let page = var(env_name).unwrap();
    let (ptr, len) = (page.as_ptr(), page.len());
    unsafe { 
        forget(page);
        transmute(from_raw_parts(ptr, len))
    }
}