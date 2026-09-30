#![no_std]
#![no_main]
#![feature(macro_metavar_expr)]

use core::net::{Ipv4Addr, SocketAddrV4};

use rustix::{io::{read, write}, net::{AddressFamily, SocketType, connect, ipproto::TCP, socket, sockopt::Timeout}};

use crate::{http::HttpClient, str::utf8_lossy, syscalls::exit};

#[macro_use]
mod log;
mod syscalls;
mod panic;
mod essentials;
mod http;
mod errors;
mod str;

const START_CHANGES_MARKER: &[u8] = "<h1>Изменения расписания</h1>".as_bytes();
const STOP_CHANGES_MARKER: &[u8] = "<br clear=\"all\"/>".as_bytes();

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start() -> ! {
    let mut count = 0;
    loop {
        let Ok(mut request) = HttpClient::get("/novosti/news_post/izmeneniya-raspisaniya", "419.spb.ru", 1000).inspect_err(|e| {
            error!("Error: {}", e);
        }) else {
            continue;
        };
        let mut buf = [0u8; 1024 * 512];
        let Ok(len) = request.plain_text(&mut buf, Some(START_CHANGES_MARKER), Some(STOP_CHANGES_MARKER)).inspect_err(|e| {
            error!("Error: {}", e);
        }) else {
            continue;
        };
        match utf8_lossy(&mut buf[..len]) {
            Ok(_) => {
                count += 1;
                info!("Successfuly parsed {}th page", count);
            },
            Err(e) => info!("Parsing error: {}", e)
        }
    };
    exit(0);
}