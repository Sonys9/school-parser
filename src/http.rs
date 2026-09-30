use core::{net::{Ipv4Addr, SocketAddrV4}, time::Duration};

use rustix::{fd::OwnedFd, io::{IoSlice, read, write, writev}, net::{AddressFamily, SocketType, connect, ipproto::TCP, socket, sockopt::{Timeout, set_socket_timeout}}};

use crate::errors::Error;

pub struct HttpClient {
    lessons_change_page: &'static str,
    lessons_page: &'static str,
}

impl HttpClient {
    pub fn new(lessons_change_page: &'static str, lessons_page: &'static str) -> Self {
        Self {
            lessons_change_page,
            lessons_page,
        }
    }

    pub fn get(path: &str, target: &str, timeout: u64) -> Result<Request, Error> {
        let fd = socket(AddressFamily::INET, SocketType::STREAM, Some(TCP))?;
        set_socket_timeout(&fd, Timeout::Send, Some(Duration::from_millis(timeout)))?;
        set_socket_timeout(&fd, Timeout::Recv, Some(Duration::from_millis(timeout)))?;
        let server_ip = Ipv4Addr::new(127, 0, 0, 1);
        let server_addr = SocketAddrV4::new(server_ip, 4190);
        connect(&fd, &server_addr)?;
        let buffers = [
            IoSlice::new(b"GET "),
            IoSlice::new(path.as_bytes()),
            IoSlice::new(b" HTTP/1.1\r\nHost: localhost:4190\r\nx-target: "),
            IoSlice::new(target.as_bytes()),
            IoSlice::new(b"\r\n\r\n"),  
        ];
        writev(&fd, &buffers)?;
        Ok(Request { fd, is_readed: false })
    }
}

pub struct Request {
    pub fd: OwnedFd,
    is_readed: bool
}

impl Request {
    pub fn plain_text(&mut self, buf: &mut [u8], start_at: Option<&[u8]>, stop_at: Option<&[u8]>) -> Result<usize, Error> {
        self.is_readed = true;
        let mut len = 0;
        let mut started = start_at.is_none();
        loop {
            let readed = read(&self.fd, &mut buf[len..]).map_err(|_| Error::Response("Timeout"))?;
            if readed == 0 {
                if len >= 4 && &buf[len - 4..] == b"\r\n\r\n" {
                    len -= 4;
                };
                break;
            };
            if let Some(start_at) = start_at {
                if let Some(index) = try_find(&buf, len, readed, start_at) && !started {
                    buf.copy_within(index..readed, 0);
                    len += readed - index;
                    started = true;
                    continue;
                };
            };
            if !started {
                continue;
            };
            if let Some(stop_at) = stop_at {
                if let Some(index) = try_find(&buf, len, readed, stop_at) {
                    len = index;
                    break;
                };
            };
            len += readed;
        };
        Ok(len)
    }
}

fn try_find(buf: &[u8], len: usize, readed: usize, query: &[u8]) -> Option<usize> {
    buf[..len + readed].windows(query.len()).position(|win| win == query)
}