use std::string::FromUtf8Error;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum UtilReadError {
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("UTF-8 Parse Error: {0}")]
    Utf8ParseError(#[from] FromUtf8Error),
}
