use crate::errors::Error;

pub fn utf8_lossy(bytes: &mut [u8], rec: bool) -> Result<&str, Error> {
    match str::from_utf8(bytes) {
        Ok(str) => Ok(str),
        Err(e) => {
            let len = e.valid_up_to();
            if len == 0 && rec {
                return Err(Error::Utf8);
            };
            bytes[len] = b'?';
            utf8_lossy(bytes, true)
        }
    }
}

fn trim_indexes<'a>(by: &'a [u8], using: &[u8]) -> (usize, usize) {
    let (mut start, mut end) = (0, by.len());
    while start < end && using.contains(&by[start]) {
        start += 1;
    }
    while end > start && using.contains(&by[end - 1]) {
        end -= 1;
    }
    (start, end)
}

pub fn trim_mut<'a>(by: &'a mut [u8], using: &[u8]) -> &'a mut [u8] {
    let (start, end) = trim_indexes(by, using);
    &mut by[start..end]
}

pub fn trim<'a>(by: &'a [u8], using: &[u8]) -> &'a [u8] {
    let (start, end) = trim_indexes(by, using);
    &by[start..end]
}
