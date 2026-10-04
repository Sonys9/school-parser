#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Request error: {0}")]
    Request(#[from] rustix::io::Errno),

    #[error("Response error: {0}")]
    Response(&'static str),

    #[error("Utf8 parse error")]
    Utf8,

    #[error("Ring is full: {0}")]
    Push(#[from] rustix_uring::squeue::PushError),
}
