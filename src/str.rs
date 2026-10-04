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