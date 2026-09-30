use rustix::io;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Request error: {0}")]
    Request(#[from] io::Errno),

    #[error("Response error: {0}")]
    Response(&'static str),

    #[error("Utf8 parse error")]
    Utf8,
}
