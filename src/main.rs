use std::{
    env::var,
    mem::{forget, transmute},
    slice::from_raw_parts,
};

use dhat::{Alloc, Profiler};
use tokio::time::Instant;
use tracing::info;

use crate::{
    http::{HttpClient, Str},
    parsers::{changes, lessons},
};

mod errors;
mod http;
mod parsers;

#[global_allocator]
static ALLOC: Alloc = Alloc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().unwrap();

    let lessons_change_page = unsafe { env("LESSONS_CHANGES_PAGE") };
    let http_client = HttpClient::new(lessons_change_page, "");

    let document = unsafe {
        Str::new(
            http_client
                .get_page_document(lessons_change_page)
                .await
                .unwrap(),
        )
    }
    .unwrap();

    let profiler = Profiler::new_heap();
    let time = Instant::now();

    let parser = changes::Parser::new(&document);
    let changes = parser.changes();
    drop(profiler);

    info!(
        "Parsed changes: {:?} in {}ns",
        changes,
        time.elapsed().as_nanos()
    );

    for change in changes {
        println!("{}", change.title);
        for row in change.change_data {
            for block in row {
                print!("{}\t", block);
            }
            println!();
        }
    }

    let content = unsafe {
        http_client
            .get_page_document("https://419.spb.ru/f/2026-deti_s_28_sent.xls")
            .await
    }
    .unwrap();

    let time = Instant::now();
    let mut lessons = lessons::Parser::new();
    unsafe { lessons.reparse(&content).unwrap() };
    lessons.get_by_class("8а");
    info!("Parsed lessons in {}ns", time.elapsed().as_nanos());
    drop(content);
}

unsafe fn env(env_name: &'static str) -> &'static str {
    let page = var(env_name).unwrap();
    let (ptr, len) = (page.as_ptr(), page.len());
    unsafe {
        forget(page);
        transmute(from_raw_parts(ptr, len))
    }
}
