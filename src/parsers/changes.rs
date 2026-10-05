use core::slice::from_raw_parts_mut;

use crate::{BigBuffer, str::{trim, trim_mut, utf8_lossy}};

pub const COLGROUP_END: &[u8] = b"</colgroup>";
pub const COLGROUP_START: &[u8] = b"<colgroup>";
pub const COL_ELEMENT: &[u8] = b"<col";
pub const SPAN_KEY: &[u8] = b"span";
pub const TD_END: &[u8] = b"</td>";
pub const TD_START: &[u8] = b"<td";
pub const WHITESPACES: &[u8] = &[b'\n', b'\r', b'\t', b'\n'];
pub const CHANGES_MARKER: &[u8] = b"color:red";
pub const START_CHANGES_MARKER: &[u8] = "<h1>Изменения расписания</h1>".as_bytes();
pub const STOP_CHANGES_MARKER: &[u8] = "<br clear=\"all\"/>".as_bytes();
pub const DELIMITER: &str = "\r\n";
pub const DELIMITER_LEN: usize = "\r\n".len();

pub struct Parser;

impl Parser {
    pub fn get_element<'a>(big_buffer: &'a mut BigBuffer, start: &'a [u8], end: &'a [u8]) -> Option<(&'a mut [u8], usize, usize)> {
        let Some(start_pos) = big_buffer.buffer.windows(start.len()).position(|bytes| bytes == start) else {
            return None;
        };
        let offset = start_pos + start.len();
        let Some(end_pos) = big_buffer.buffer[offset..].windows(end.len()).position(|bytes| bytes == end) else {
            return None;
        };
        Some((&mut big_buffer.buffer[offset..offset + end_pos], start_pos, end_pos + offset))
    }

    pub fn colgroup(big_buffer: &mut BigBuffer) -> Option<u8> {
        let Some((colgroup, _, end_pos)) = Self::get_element(big_buffer, COLGROUP_START, COLGROUP_END) else {
            return None;
        };
        let mut count = 0;
        for line in colgroup.split(|&byte| byte == b'\n') {
            let clean = trim(line, WHITESPACES);
            if !clean.starts_with(COL_ELEMENT) {
                continue;
            };
            let mut columns = 1;
            for mut pair in clean.split(|&byte| byte == b' ').map(|bytes| bytes.split(|&byte| byte == b'=')) {
                let Some(key) = pair.next().map(|key| trim(key, &[b'"'])) else { continue };
                let Some(value) = pair.next().map(|key| trim(key, &[b'"'])) else { continue };
                if key != SPAN_KEY {
                    continue;
                };
                if value.len() != 1 {
                    break;
                };
                columns = value[0] - b'0';
            };
            count += columns;
        };
        let start = end_pos + COLGROUP_END.len();
        if start > big_buffer.len {
            return None;
        };
        big_buffer.move_buffer(start, 0);
        Some(count)
    }
    
    pub fn title_fragment(big_buffer: &mut BigBuffer) -> Option<&mut [u8]> {
        loop {
            let Some(index) = big_buffer.buffer[..big_buffer.len].iter().position(|&byte| byte == b'\n') else {
                break;
            };
            let line = &mut big_buffer.buffer[..index];
            if line.windows(CHANGES_MARKER.len()).position(|bytes| bytes == CHANGES_MARKER).is_none() {
                big_buffer.move_buffer(index + 1, 0);
                continue;
            };
            let Some((slice, start_pos, _)) = Self::get_element(big_buffer, b"text-decoration:none\">", b"<") else {
                break;
            };
            let len = slice.len();
            big_buffer.move_buffer(start_pos + b"text-decoration:none\">".len(), 0);
            return Some(&mut big_buffer.buffer[..len]);
        };
        None
    }

    #[inline]
    pub fn find(buffer: &[u8], what: &[u8]) -> Option<usize> {
        buffer.windows(what.len())
            .position(|bytes| bytes == what)
    }
}