#![no_std]
#![no_main]
#![feature(macro_metavar_expr)]

use core::{arch::asm, net::{Ipv4Addr, SocketAddrV4}};

use rustix::{fd::AsRawFd, io::{read, write}, net::{AddressFamily, SocketType, connect, ipproto::TCP, socket, sockopt::Timeout}};
use rustix_uring::{opcode, types};

use crate::{http::HttpClient, str::utf8_lossy, syscalls::{align, exit}};

#[macro_use]
mod log;
mod syscalls;
mod panic;
mod essentials;
mod http;
mod errors;
mod str;
mod telegram;

const START_CHANGES_MARKER: &[u8] = "<h1>Изменения расписания</h1>".as_bytes();
const STOP_CHANGES_MARKER: &[u8] = "<br clear=\"all\"/>".as_bytes();

#[allow(static_mut_refs)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn main() {
    let mut client = HttpClient::new("", "");
    let Ok(mut request) = client.get("/novosti/news_post/izmeneniya-raspisaniya", "419.spb.ru", 1000).inspect_err(|e| {
        error!("Error: {}", e);
    }) else {
        exit(1);
    };
    client.wait().unwrap();
    info!("Sent");
    let cqe = client.ring.completion().next().unwrap();
    info!("Sent {:?} bytes", cqe.result().unwrap());
    let mut buffer = [0u8; 1024];
    let mut is_changes = false;
    let mut big_buffer = BigBuffer::new();
    loop {
        request.next(&mut buffer, &mut client.ring).unwrap();
        client.wait().unwrap();
        let cqe = client.ring.completion().next().unwrap();
        let len = cqe.result().unwrap() as usize;
        if len == 0 {
            break;
        };
        if buffer.ends_with(b"0\r\n\r\n") {
            break;
        };
        big_buffer.update(buffer, len);
        if let Some(start_index) = big_buffer.buffer
            .windows(START_CHANGES_MARKER.len())
            .position(|bytes| bytes == START_CHANGES_MARKER)
        {
            big_buffer.move_buffer(start_index);
            is_changes = true;
        };
        if is_changes {
            terminal!("{}", utf8_lossy(&mut big_buffer.buffer[..big_buffer.first_len], false).unwrap());
        };

        if let Some(stop_index) = big_buffer.buffer
            .windows(STOP_CHANGES_MARKER.len())
            .position(|bytes| bytes == STOP_CHANGES_MARKER)
        {
            big_buffer.second_len = stop_index;
            is_changes = false;
        };

    };
}

pub struct BigBuffer {
    pub buffer: [u8; 1024 * 2],
    pub first_len: usize,
    pub second_len: usize,
}

impl BigBuffer {
    pub fn new() -> Self {
        Self {
            buffer: [0u8; 1024 * 2],
            first_len: 0,
            second_len: 0,
        }
    }

    #[inline]
    pub fn update(&mut self, buffer: [u8; 1024], len: usize) {
        self.move_buffer(1024);
        self.buffer[1024..].copy_from_slice(&buffer);
        self.first_len = self.second_len;
        self.second_len = len;
    }

    #[inline]
    pub fn move_buffer(&mut self, index: usize) {
        self.buffer.copy_within(index.., 0);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start() -> ! {
    align(main);
    exit(0);
}