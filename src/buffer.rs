use crate::parsers::changes::{DELIMITER, DELIMITER_LEN};

pub const TCP_BUFFER_LEN: usize = 1024 - 128;

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

    pub fn update(&mut self, buffer: [u8; TCP_BUFFER_LEN], len: usize) {
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
            for (i, bytes) in self.buffer[index + DELIMITER_LEN..self.len]
                .windows(2)
                .enumerate()
            {
                if bytes == DELIMITER.as_bytes() {
                    stop_index = Some(i);
                    break;
                };
            }
            let Some(stop_index) = stop_index else {
                break;
            };
            let total_len = DELIMITER_LEN + stop_index + DELIMITER_LEN;
            self.move_buffer(index + total_len, index);
        }
    }

    #[inline]
    fn get_delimiter(&mut self) -> Option<usize> {
        self.buffer[..self.len]
            .windows(2)
            .position(|bytes| bytes == b"\r\n")
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
    }
}
