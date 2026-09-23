#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] std::str::Utf8Error),
}
