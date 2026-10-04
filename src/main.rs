#![no_std]
#![no_main]
#![feature(macro_metavar_expr)]

use core::{arch::asm, net::{Ipv4Addr, SocketAddrV4}};

use rustix::{fd::AsRawFd, io::{read, write}, net::{AddressFamily, SocketType, connect, ipproto::TCP, socket, sockopt::Timeout}};
use rustix_uring::{opcode, types};

use crate::{http::HttpClient, parsers::html, str::utf8_lossy, syscalls::{align, exit}};

#[macro_use]
mod log;
mod syscalls;
mod panic;
mod essentials;
mod http;
mod errors;
mod str;
mod telegram;
mod parsers;

const START_CHANGES_MARKER: &[u8] = "<h1>Изменения расписания</h1>".as_bytes();
const STOP_CHANGES_MARKER: &[u8] = "<br clear=\"all\"/>".as_bytes();

#[allow(static_mut_refs)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn main() -> ! {
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
    let mut state = State::NotReady;
    let mut big_buffer = BigBuffer::new();
    loop {
        request.next(&mut buffer, &mut client.ring).unwrap();
        client.wait().unwrap();
        let cqe = client.ring.completion().next().unwrap();
        let len = cqe.result().unwrap() as usize;
        if len == 0 || buffer[..len].ends_with(b"0\r\n\r\n") {
            break;
        };

        big_buffer.update(buffer, len);
        if let Some(start_index) = big_buffer.buffer
            .windows(START_CHANGES_MARKER.len())
            .position(|bytes| bytes == START_CHANGES_MARKER)
        {
            big_buffer.move_buffer(start_index);
            state = State::Ready;
        };

        if let Some(stop_index) = big_buffer.buffer
            .windows(STOP_CHANGES_MARKER.len())
            .position(|bytes| bytes == STOP_CHANGES_MARKER)
        {
            big_buffer.update_lengths(stop_index);
            state = State::Ended;
        };

        if state == State::NotReady {
            continue;
        };

        terminal!("{:?}\n\n", utf8_lossy(&mut buffer[..len], false).unwrap());
        //terminal!("{:?}\n", utf8_lossy(&mut big_buffer.buffer[1024..1024 + big_buffer.second_len], false).unwrap());
        // if let Some(columns) = html::Parser::colgroup(&mut big_buffer) {
            //info!("Columns: {}", columns);
        // };

        if state == State::Ended {
            break;
        };
    };
    exit(0);
}

#[derive(PartialEq)]
pub enum State {
    NotReady,
    Ready,
    Ended,
}

pub struct BigBuffer {
    pub buffer: [u8; 1024 * 2],
    pub first_len: usize,
    pub second_len: usize,
    pub total_len: usize,
    empty: bool,
}

impl BigBuffer {
    pub fn new() -> Self {
        Self {
            buffer: [0u8; 1024 * 2],
            first_len: 0,
            second_len: 0,
            total_len: 0,
            empty: true,
        }
    }

    #[inline]
    pub fn update(&mut self, buffer: [u8; 1024], len: usize) {
        self.move_buffer(1024); // TODO remove every len string like ..\r\n1000\r\n...
        self.buffer[1024..].copy_from_slice(&buffer);
        let mut index = 0;
        for (i,  bytes) in self.buffer[1024..].windows("\r\n".len()).enumerate() {
            if bytes == b"\r\n" {

            };
        };
        self.first_len = self.second_len;
        self.second_len = len;
        if self.empty {
            self.move_buffer(1024);
            self.empty = false;
            self.second_len = 0;
        };
    }

    #[inline]
    pub fn move_buffer(&mut self, index: usize) {
        self.buffer.copy_within(index.., 0);
        self.update_lengths(1024 * 2 - index);
    }

    #[inline]
    pub fn update_lengths(&mut self, total_len: usize) {
        self.total_len = total_len;
        (self.first_len, self.second_len) = if self.total_len <= 1024 {
            (self.total_len, 0)
        } else {
            (1024, self.total_len - 1024)
        };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start() -> ! {
    align(main);
    exit(0);
}