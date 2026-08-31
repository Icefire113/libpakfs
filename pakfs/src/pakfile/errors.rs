use std::io;
use thiserror::Error;

use crate::serialization::errors::DeSerError;

#[derive(Debug, Error)]
pub enum PakFileError {
    #[error("IO Error: {0}")]
    IoError(#[from] io::Error),
    #[error("Deserialization Error: {0}")]
    DeSerError(#[from] DeSerError),
}
