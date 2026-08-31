use std::io;

use thiserror::Error;

/// Errors produced when building, reading, or writing pak files.
#[derive(Debug, Error)]
pub enum PakError {
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),

    #[error("Path not found in pak file: {0}")]
    NotFound(String),

    #[error("Buffer too small: needed {needed} bytes, got {got}")]
    BufferTooSmall { needed: u64, got: usize },

    #[error("Invalid file magic, not a pak file")]
    BadMagic,

    #[error("Malformed pak file: {0}")]
    Malformed(&'static str),

    #[error("Unknown compression codec id: {0}")]
    UnknownCodec(u8),

    #[error("Duplicate path: {0}")]
    DuplicatePath(String),

    #[error("Compression error: {0}")]
    Compression(String),
}
