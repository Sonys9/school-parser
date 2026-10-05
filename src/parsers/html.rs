use crate::{BigBuffer, str::utf8_lossy};

const COLGROUP_END: &[u8] = b"</colgroup>";
const COLGROUP_START: &[u8] = b"<colgroup>";
const COL_ELEMENT: &[u8] = b"<col";


pub struct Parser {
    
}

impl Parser {
    pub fn colgroup(big_buffer: &mut BigBuffer) -> Option<usize> {
        let Some(start_pos) = big_buffer.buffer.windows(COLGROUP_START.len()).position(|bytes| bytes == COLGROUP_START) else {
            return None;
        };
        let Some(end_pos) = big_buffer.buffer.windows(COLGROUP_END.len()).position(|bytes| bytes == COLGROUP_END) else {
            return None;
        };
        let colgroup = &big_buffer.buffer[start_pos + COLGROUP_START.len()..end_pos];
        let mut count = 0;
        for bytes in colgroup.windows(COL_ELEMENT.len()) {
            if bytes == COL_ELEMENT {
                count += 1;
            };
        };
        let start = end_pos + COLGROUP_END.len();
        if start > big_buffer.len {
            return None;
        };
        info!("moving {} to {}", start, 0);
        big_buffer.move_buffer(start, 0);
        info!("new len: {}", big_buffer.len);
        Some(count)
    }
}