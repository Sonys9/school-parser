#![no_std]
#![no_main]
#![feature(macro_metavar_expr)]

use core::{arch::asm, net::{Ipv4Addr, SocketAddrV4}, ops::RangeBounds};

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
const DELIMITER: &str = "\r\n";
const DELIMITER_LEN: usize = "\r\n".len();

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
    let mut is_headers = true;
    loop {
        request.next(&mut buffer, &mut client.ring).unwrap();
        client.wait().unwrap();
        let cqe = client.ring.completion().next().unwrap();
        let len = cqe.result().unwrap() as usize;
        if len == 0 || buffer[..len].ends_with(b"0\r\n\r\n") {
            break;
        };
        
        big_buffer.update(buffer, len);

        if is_headers {
            for line in buffer[..len].split(|&byte| byte == b'\n') {
                if line == b"\r" {
                    is_headers = false;
                    break;
                };
                let Ok(line) = str::from_utf8(line) else {
                    continue;
                };
                let mut header = line.split(": ");
                let (Some(name), Some(value)) = (header.next(), header.next()) else {
                    continue;
                };
                if name != "Transfer-Encoding" {
                    continue;
                };
                info!("Transfer encoding is {}", value);
                if value != "chunked\r" {
                    break;
                };
                big_buffer.is_chunked = true;
            }
        };

        if let Some(start_index) = big_buffer.buffer[..big_buffer.len]
            .windows(START_CHANGES_MARKER.len())
            .position(|bytes| bytes == START_CHANGES_MARKER)
        {
            big_buffer.move_buffer(start_index, 0);
            state = State::Ready;
        };

        if let Some(stop_index) = big_buffer.buffer[..big_buffer.len]
            .windows(STOP_CHANGES_MARKER.len())
            .position(|bytes| bytes == STOP_CHANGES_MARKER)
        {
            big_buffer.len = stop_index;
            state = State::Ended;
        };

        if state == State::NotReady {
            continue;
        };

        terminal!("{:?}\n\n", utf8_lossy(&mut big_buffer.buffer[..big_buffer.len], false).unwrap());
        //terminal!("{:?}\n", stringify);
        
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
    pub len: usize,
    pub is_chunked: bool,
}

impl BigBuffer {
    pub fn new() -> Self {
        Self {
            buffer: [0u8; 1024 * 2],
            len: 0,
            is_chunked: false,
        }
    }

    pub fn update(&mut self, buffer: [u8; 1024], len: usize) {
        let clean_buffer = &buffer[..len];
        self.len = if 2048 - self.len < len {
            let diff = self.len + len - 2048;
            let old_len = self.len - diff;
            self.move_buffer(diff, 0);
            self.buffer[old_len..old_len + len].copy_from_slice(clean_buffer);
            2048
        } else {
            let sum = self.len + len;
            self.buffer[self.len..sum].copy_from_slice(clean_buffer);
            sum
        };

        if !self.is_chunked {
            return;
        };
        let mut is_size = false;
        while let Some(index) = self.get_delimiter() {
            is_size = !is_size;
            let mut stop_index = None;
            for (i, bytes) in self.buffer[index + DELIMITER_LEN..self.len].windows(2).enumerate() {
                if bytes == DELIMITER.as_bytes() {
                    stop_index = Some(i);
                    break;
                };
            };
            let Some(stop_index) = stop_index else {
                break;
            };
            let total_len = DELIMITER_LEN + stop_index + DELIMITER_LEN;
            self.move_buffer(index + total_len, index);
        };

        /*
        self.move_buffer(1024, 0); // TODO remove every len string like ..\r\n1000\r\n...
        self.buffer[1024..].copy_from_slice(&buffer);

        let mut index = 0;
        self.first_len = self.second_len;
        self.second_len = len;
        if self.empty {
            self.move_buffer(1024, 0);
            self.empty = false;
            self.second_len = 0;
        };
        */
    }

    #[inline]
    fn get_delimiter(&mut self) -> Option<usize> {
        self.buffer[..self.len].windows(2).position(|bytes| bytes == b"\r\n")
    }

    #[inline]
    pub fn remove(&mut self, start: usize, end: usize) {
        if start > end {
            panic!("Can't remove due to start > end!");
        };
        self.len = self.len - (end - start);
        self.move_buffer(end, start);
    }

    #[inline]
    pub fn move_buffer(&mut self, index: usize, dest: usize) {
        self.buffer.copy_within(index.., dest);
        self.len -= index - dest;
        // self.update_lengths(1024 * 2 - index);
    }

    /*
    #[inline]
    pub fn update_lengths(&mut self, total_len: usize) {
        self.total_len = total_len;
        (self.first_len, self.second_len) = if self.total_len <= 1024 {
            (self.total_len, 0)
        } else {
            (1024, self.total_len - 1024)
        };
    }
    */
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start() -> ! {
    align(main);
    exit(0);
}