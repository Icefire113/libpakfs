use crate::serialization::pakfile::PAKFILE_VERSION;
use crate::util;

use std::io;
use thiserror::Error;

/// This represents an error that occured when deserializing a pak file
#[derive(Debug, Error)]
pub enum DeSerError {
    #[error("IO Error: {0}")]
    IoError(#[from] io::Error),

    #[error("Read Error: {0}")]
    ReadError(#[from] util::errors::UtilReadError),

    #[error("Invalid file magic!")]
    InvalidFileMagic,

    #[error(
        "This pak file cannot be read with this version of libpakfs, \
        pak file version: {actual}, libpakfs version: {PAKFILE_VERSION}"
    )]
    InvalidFileVersion { actual: u32 },
}

#[derive(Debug, Error)]
pub enum FileSaveError {
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum SerializerError {
    #[error("IO Error: {0}")]
    IoError(#[from] io::Error),

    #[error("Unable to write to file")]
    UnwritableError,

    #[error("Is not a file")]
    NotAFileError,
}
