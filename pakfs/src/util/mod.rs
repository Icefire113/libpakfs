pub(crate) mod errors;

use std::{
    fs::File,
    io::{self, BufRead, Read, Seek},
};

use crate::util::{self, errors::UtilReadError};

#[allow(dead_code)]
/// Reads `n` bytes from the reader
pub(crate) fn read_n_bytes(
    reader: &mut std::io::BufReader<&File>,
    n: usize,
) -> Result<Vec<u8>, UtilReadError> {
    let mut buff = vec![0u8; n];
    reader.read_exact(&mut buff)?;
    Ok(buff)
}

#[allow(dead_code)]
/// Reads a u8 from the reader
pub(crate) fn read_u8(reader: &mut std::io::BufReader<&File>) -> Result<u8, UtilReadError> {
    let mut buff = [0u8; size_of::<u8>()];
    reader.read_exact(&mut buff)?;
    Ok(u8::from_le_bytes(buff))
}

#[allow(dead_code)]
/// Reads a u16 from the reader
pub(crate) fn read_u16(reader: &mut std::io::BufReader<&File>) -> Result<u16, UtilReadError> {
    let mut buff = [0u8; size_of::<u16>()];
    reader.read_exact(&mut buff)?;
    Ok(u16::from_le_bytes(buff))
}

#[allow(dead_code)]
/// Reads a u32 from the reader
pub(crate) fn read_u32(reader: &mut std::io::BufReader<&File>) -> Result<u32, UtilReadError> {
    let mut buff = [0u8; size_of::<u32>()];
    reader.read_exact(&mut buff)?;
    Ok(u32::from_le_bytes(buff))
}

#[allow(dead_code)]
/// Reads a u64 from the reader
pub(crate) fn read_u64(reader: &mut std::io::BufReader<&File>) -> Result<u64, UtilReadError> {
    let mut buff = [0u8; size_of::<u64>()];
    reader.read_exact(&mut buff)?;
    Ok(u64::from_le_bytes(buff))
}

#[allow(dead_code)]
/// Reads a C style string from the reader
pub(crate) fn read_string(reader: &mut std::io::BufReader<&File>) -> Result<String, UtilReadError> {
    let mut buff = vec![];
    reader.read_until('\0' as u8, &mut buff)?;
    Ok(String::from_utf8(buff)?)
}

#[allow(dead_code)]
/// Reads until a 2 byte pattern is read, returning the bytes read including the pattern
pub(crate) fn read_until_bytes<R: Read>(
    reader: &mut R,
    pattern: [u8; 2],
) -> Result<Vec<u8>, UtilReadError> {
    let mut buff: Vec<u8> = Vec::new();
    let mut window = [0u8; 2];
    let mut window_pos = 0;
    let mut byte = [0u8; 1];

    // fill starting window
    for i in 0..2 {
        match reader.read_exact(&mut byte) {
            Ok(()) => {
                window[i] = byte[0];
                buff.push(byte[0]);
            }
            Err(e) => {
                return Err(util::errors::UtilReadError::IoError(e));
            }
        }
    }

    if window == pattern {
        return Ok(buff);
    }

    loop {
        reader.read_exact(&mut byte)?;
        buff.push(byte[0]);

        window[window_pos] = byte[0];
        window_pos = (window_pos + 1) % 2;

        if window[window_pos] == pattern[0] && window[(window_pos + 1) % 2] == pattern[1] {
            break;
        }
    }

    Ok(buff)
}

#[allow(dead_code)]
/// Skips `n` bytes in the reader
pub(crate) fn skip<R: Read + Seek>(reader: &mut R, n: u64) -> io::Result<()> {
    reader.seek_relative(n as i64)?;
    Ok(())
}
