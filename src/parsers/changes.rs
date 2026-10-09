use core::slice::from_raw_parts_mut;

use crate::{
    BigBuffer,
    str::{trim, trim_mut, utf8_lossy},
};

pub const COLGROUP_END_MARKER: &[u8] = b"</colgroup>";
pub const COLGROUP_START_MARKER: &[u8] = b"<colgroup>";
pub const COL_ELEMENT_MARKER: &[u8] = b"<col";
pub const SPAN_KEY: &[u8] = b"span";
pub const TD_END_MARKER: &[u8] = b"</td>";
pub const TD_START_MARKER: &[u8] = b"<td";
pub const TR_START_MARKER: &[u8] = b"<tr";
pub const TBODY_START_MARKER: &[u8] = b"<tbody>";
pub const WHITESPACES: &[u8] = &[b'\n', b'\r', b'\t', b'\n'];
pub const CHANGES_MARKER: &[u8] = b"color:red";
pub const START_CHANGES_MARKER: &[u8] = "<h1>Изменения расписания</h1>".as_bytes();
pub const STOP_CHANGES_MARKER: &[u8] = "<br clear=\"all\"/>".as_bytes();
pub const DELIMITER: &str = "\r\n";
pub const DELIMITER_LEN: usize = "\r\n".len();
pub const TEXT_START_MARKER: &[u8] = b">";
pub const TEXT_CLOSE_MARKER: &[u8] = b"</span></span></span></span>";

pub struct Parser;

impl Parser {
    pub fn get_element<'a>(
        big_buffer: &'a mut BigBuffer,
        start: &'a [u8],
        end: &'a [u8], 
    ) -> Option<(&'a mut [u8], usize, usize)> {
        let end_pos = big_buffer.buffer[..big_buffer.len]
            .windows(end.len())
            .position(|bytes| bytes == end)?;
        let start_pos = big_buffer.buffer[..end_pos]
            .windows(start.len())
            .rposition(|bytes| bytes == start)?;
        Some((
            &mut big_buffer.buffer[start_pos + start.len()..end_pos],
            start_pos,
            end_pos,
        ))
    }

    pub fn colgroup(big_buffer: &mut BigBuffer) -> Option<u8> {
        let (colgroup, _, end_pos) =
            Self::get_element(big_buffer, COLGROUP_START_MARKER, COLGROUP_END_MARKER)?;
        let mut count = 0;
        for line in colgroup.split(|&byte| byte == b'\n') {
            let clean = trim(line, WHITESPACES);
            if !clean.starts_with(COL_ELEMENT_MARKER) {
                continue;
            }
            let mut columns = 1;
            for mut pair in clean
                .split(|&byte| byte == b' ')
                .map(|bytes| bytes.split(|&byte| byte == b'='))
            {
                let Some(key) = pair.next().map(|key| trim(key, &[b'"'])) else {
                    continue;
                };
                let Some(value) = pair.next().map(|key| trim(key, &[b'"'])) else {
                    continue;
                };
                if key != SPAN_KEY {
                    continue;
                };
                if value.len() != 1 {
                    break;
                };
                columns = value[0] - b'0';
            }
            count += columns;
        }
        let start = end_pos + COLGROUP_END_MARKER.len();
        if start > big_buffer.len {
            return None;
        };
        big_buffer.move_buffer(start, 0);
        Some(count)
    }

    pub fn extract_span(big_buffer: &mut BigBuffer) -> Option<(&mut [u8], usize)> {
        loop {
            let (_, start_pos, end_pos) =
                Self::get_element(big_buffer, TEXT_START_MARKER, TEXT_CLOSE_MARKER)?;
            let len = end_pos - (start_pos + TEXT_START_MARKER.len());
            big_buffer.move_buffer(start_pos + TEXT_START_MARKER.len(), 0);
            return Some((&mut big_buffer.buffer[..len], start_pos));
        }
    }

    #[inline]
    pub fn find(buffer: &[u8], what: &[u8]) -> Option<usize> {
        buffer.windows(what.len()).position(|bytes| bytes == what)
    }
}
