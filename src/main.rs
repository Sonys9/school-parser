#![no_std]
#![no_main]
#![feature(macro_metavar_expr)]

use core::{
    arch::asm,
    net::{Ipv4Addr, SocketAddrV4},
    ops::RangeBounds,
};

use rustix::{
    fd::AsRawFd,
    io::{read, write},
    net::{AddressFamily, SocketType, connect, ipproto::TCP, socket, sockopt::Timeout},
    time,
};
use rustix_uring::{opcode, types};

use crate::{
    buffer::BigBuffer, http::{HttpClient, Request}, parsers::changes::{
        self, CHANGES_MARKER, START_CHANGES_MARKER, STOP_CHANGES_MARKER, TBODY_START_MARKER, TD_END_MARKER, TD_START_MARKER, TEXT_CLOSE_MARKER, TR_START_MARKER, WHITESPACES,
    }, str::{trim, trim_mut, utf8_lossy}, syscalls::{align, exit, timestamp},
};

#[macro_use]
mod log;
mod buffer;
mod errors;
mod essentials;
mod http;
mod panic;
mod parsers;
mod str;
mod syscalls;
mod telegram;

fn main() -> ! {
    let mut client = HttpClient::new("", "");
    let Ok(mut request) = client
        .get(
            "/novosti/news_post/izmeneniya-raspisaniya",
            "419.spb.ru",
            1000,
        )
        .inspect_err(|e| {
            error!("Error: {}", e);
        })
    else {
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
    let mut start_time = None;
    let mut total_time = 0;
    let mut tbody_id = 0;
    let mut tr_id = 0;
    loop {
        if let Some(start_time) = start_time {
            total_time += timestamp() - start_time;
        };
        request.next(&mut buffer, &mut client.ring).unwrap();
        client.wait().unwrap();
        let cqe = client.ring.completion().next().unwrap();
        let len = cqe.result().unwrap() as usize;
        if len == 0 || buffer[..len].ends_with(b"0\r\n\r\n") {
            break;
        };
        start_time = Some(timestamp());

        // terminal!("===========================\n{}", utf8_lossy(&mut buffer[..len], false).unwrap());
        big_buffer.update(buffer, len);

        if is_headers && let (Some(encoding), is_ended) = Request::encoding_type(&buffer) {
            big_buffer.is_chunked = encoding == "chunked\r";
            is_headers = !is_ended;
        };

        if let Some(start_index) =
            changes::Parser::find(&big_buffer.buffer[..big_buffer.len], START_CHANGES_MARKER)
        {
            big_buffer.move_buffer(start_index, 0);
            state = State::Ready;
        };

        if let Some(stop_index) =
            changes::Parser::find(&big_buffer.buffer[..big_buffer.len], STOP_CHANGES_MARKER)
        {
            big_buffer.len = stop_index;
            state = State::Ended;
        };

        if state == State::NotReady {
            continue;
        };

        if let Some(columns) = changes::Parser::colgroup(&mut big_buffer) {
            info!("Columns: {}", columns);
        };
        if let Some(position) = big_buffer.buffer[..big_buffer.len]
            .windows(TBODY_START_MARKER.len())
            .position(|bytes| bytes == TBODY_START_MARKER)
        {
            big_buffer.move_buffer(position + TBODY_START_MARKER.len(), 0);
            tbody_id += 1;
            tr_id = 0;
        };

        let mut tr_pos = None;
        loop {
            if let Some(position) = big_buffer.buffer[..big_buffer.len]
                .windows(TR_START_MARKER.len())
                .position(|bytes| bytes == TR_START_MARKER)
            {
                tr_pos = Some(position);
            };

            // info!("{}", utf8_lossy(&mut big_buffer.buffer[..big_buffer.len], false).unwrap());
            let Some((text, start_pos)) = changes::Parser::extract_span(&mut big_buffer) else {
                break;
            };
            if let Some(position) = tr_pos {
                if position < start_pos {
                    tr_id += 1;
                    tr_pos = None;
                };
            };
            info!(
                "[{} at tr {}] result {}",
                tbody_id,
                tr_id,
                utf8_lossy(text, false).unwrap()
            );
            // info!("{}", utf8_lossy(&mut big_buffer.buffer[..big_buffer.len], false).unwrap());
            let text_len = text.len();
            big_buffer.move_buffer(text_len + TEXT_CLOSE_MARKER.len(), 0);
        }

        if state == State::Ended {
            break;
        };
    }
    info!("Parsed in {}ns", total_time);
    exit(0);
}

#[derive(PartialEq)]
pub enum State {
    NotReady,
    Ready,
    Ended,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start() -> ! {
    align(main);
    exit(0);
}
