use core::{
    ffi::c_void,
    net::{Ipv4Addr, SocketAddrV4},
    time::Duration,
};

use rustix::{
    fd::{AsRawFd, OwnedFd},
    io::{IoSlice, close, read, write, writev},
    io_uring::iovec,
    net::{
        AddressFamily, SocketType, connect,
        ipproto::TCP,
        socket,
        sockopt::{Timeout, set_socket_timeout},
    },
};
use rustix_uring::{IoUring, opcode, types};

use crate::errors::Error;

pub struct HttpClient {
    lessons_change_page: &'static str,
    lessons_page: &'static str,
    pub ring: IoUring,
}

impl HttpClient {
    pub fn new(lessons_change_page: &'static str, lessons_page: &'static str) -> Self {
        let ring = IoUring::new(128).expect("Failed to get the ring");
        Self {
            lessons_change_page,
            lessons_page,
            ring,
        }
    }

    pub fn get(&mut self, path: &str, target: &str, timeout: u64) -> Result<Request, Error> {
        let fd = socket(AddressFamily::INET, SocketType::STREAM, Some(TCP))?;
        set_socket_timeout(&fd, Timeout::Send, Some(Duration::from_millis(timeout)))?;
        set_socket_timeout(&fd, Timeout::Recv, Some(Duration::from_millis(timeout)))?;
        let server_ip = Ipv4Addr::new(127, 0, 0, 1);
        let server_addr = SocketAddrV4::new(server_ip, 4190);
        connect(&fd, &server_addr)?;
        let buffers = [
            iovec::new(b"GET "),
            iovec::new(path.as_bytes()),
            iovec::new(b" HTTP/1.1\r\nHost: localhost:4190\r\nx-target: "),
            iovec::new(target.as_bytes()),
            iovec::new(b"\r\n\r\n"),
        ];
        let write_e = opcode::Writev::new(
            types::Fd(fd.as_raw_fd()),
            buffers.as_ptr(),
            buffers.len() as u32,
        )
        .build();
        unsafe {
            self.ring.submission().push(&write_e)?;
        };
        Ok(Request {
            fd,
            is_readed: false,
        })
    }

    pub fn wait(&self) -> Result<usize, Error> {
        Ok(self.ring.submit_and_wait(1)?)
    }
}

pub struct Request {
    pub fd: OwnedFd,
    is_readed: bool,
}

impl Request {
    pub fn next(&mut self, buffer: &mut [u8], ring: &mut IoUring) -> Result<(), Error> {
        let read_e = opcode::Read::new(
            types::Fd(self.fd.as_raw_fd()),
            buffer.as_mut_ptr(),
            buffer.len() as u32,
        )
        .build();
        unsafe {
            ring.submission().push(&read_e)?;
        };
        Ok(())
    }

    pub fn encoding_type<'a>(buffer: &'a [u8]) -> (Option<&'a str>, bool) {
        for line in buffer.split(|&byte| byte == b'\n') {
            if line == b"\r" {
                return (None, true);
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
            return (Some(value), false);
        }
        (None, false)
    }

    /* pub fn blocking_plain_text(&mut self, buf: &mut [u8], start_at: Option<&[u8]>, stop_at: Option<&[u8]>) -> Result<usize, Error> {
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
    } */
}

fn try_find(buf: &[u8], len: usize, readed: usize, query: &[u8]) -> Option<usize> {
    buf[..len + readed]
        .windows(query.len())
        .position(|win| win == query)
}

trait New {
    fn new(data: &[u8]) -> Self;
}

impl New for iovec {
    fn new(data: &[u8]) -> Self {
        Self {
            iov_base: data.as_ptr() as *mut c_void,
            iov_len: data.len(),
        }
    }
}
