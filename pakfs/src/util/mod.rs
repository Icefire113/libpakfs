pub(crate) mod errors;

use std::io::{self, BufRead};

use crate::util::errors::UtilReadError;

#[allow(dead_code)]
/// Reads `n` bytes from the reader
pub(crate) fn read_n_bytes<R: BufRead>(reader: &mut R, n: usize) -> Result<Vec<u8>, UtilReadError> {
    let mut buff = vec![0u8; n];
    reader.read_exact(&mut buff)?;
    Ok(buff)
}

#[allow(dead_code)]
/// Reads a u8 from the reader
pub(crate) fn read_u8<R: BufRead>(reader: &mut R) -> Result<u8, UtilReadError> {
    let mut buff = [0u8; size_of::<u8>()];
    reader.read_exact(&mut buff)?;
    Ok(u8::from_le_bytes(buff))
}

#[allow(dead_code)]
/// Reads a u16 from the reader
pub(crate) fn read_u16<R: BufRead>(reader: &mut R) -> Result<u16, UtilReadError> {
    let mut buff = [0u8; size_of::<u16>()];
    reader.read_exact(&mut buff)?;
    Ok(u16::from_le_bytes(buff))
}

#[allow(dead_code)]
/// Reads a u64 from the reader
pub(crate) fn read_u64<R: BufRead>(reader: &mut R) -> Result<u64, UtilReadError> {
    let mut buff = [0u8; size_of::<u64>()];
    reader.read_exact(&mut buff)?;
    Ok(u64::from_le_bytes(buff))
}

#[allow(dead_code)]
/// Skips `n` bytes in the reader
pub(crate) fn skip<R: BufRead + io::Seek>(reader: &mut R, n: u64) -> io::Result<()> {
    reader.seek_relative(n as i64)?;
    Ok(())
}
