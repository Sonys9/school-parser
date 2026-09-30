#![no_std]
#![no_main]
#![feature(macro_metavar_expr)]

use core::net::{Ipv4Addr, SocketAddrV4};

use rustix::{io::{read, write}, net::{AddressFamily, SocketType, connect, ipproto::TCP, socket}};

use crate::syscalls::{exit, terminal, timestamp};

#[macro_use]
mod log;
mod syscalls;
mod panic;
mod essentials;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start() -> ! {
    let fd = socket(AddressFamily::INET, SocketType::STREAM, Some(TCP)).expect("Failed to connect to the socket");
    let server_ip = Ipv4Addr::new(185, 32, 58, 252);
    let server_addr = SocketAddrV4::new(server_ip, 80);
    connect(&fd, &server_addr).expect("Failed to connect to the server");
    write(&fd, b"GET / HTTP/1.1\r\nHost: 419.spb.ru\r\n\r\n").expect("Failed to send the request");
    let mut buf = [0u8; 1024];
    let len = read(&fd, &mut buf).expect("Failed to read the response");
    info!("Response: {:?}", str::from_utf8(&buf[..len]));
    exit(0);
}