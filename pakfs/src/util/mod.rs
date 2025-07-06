use std::{
    fs::File,
    io::{self, BufRead, Read},
};

#[allow(dead_code)]
/// Reads a u32 from the reader
pub(crate) fn read_u32(reader: &mut std::io::BufReader<&File>) -> Result<u32, String> {
    let mut buff = [0u8; size_of::<u32>()];
    reader.read_exact(&mut buff).map_err(|e| e.to_string())?;
    Ok(u32::from_le_bytes(buff))
}

#[allow(dead_code)]
/// Reads a u64 from the reader
pub(crate) fn read_u64(reader: &mut std::io::BufReader<&File>) -> Result<u64, String> {
    let mut buff = [0u8; size_of::<u64>()];
    reader.read_exact(&mut buff).map_err(|e| e.to_string())?;
    Ok(u64::from_le_bytes(buff))
}

#[allow(dead_code)]
/// Reads a u16 from the reader
pub(crate) fn read_u16(reader: &mut std::io::BufReader<&File>) -> Result<u16, String> {
    let mut buff = [0u8; size_of::<u16>()];
    reader.read_exact(&mut buff).map_err(|e| e.to_string())?;
    Ok(u16::from_le_bytes(buff))
}

#[allow(dead_code)]
/// Reads a u8 from the reader
pub(crate) fn read_u8(reader: &mut std::io::BufReader<&File>) -> Result<u8, String> {
    let mut buff = [0u8; size_of::<u8>()];
    reader.read_exact(&mut buff).map_err(|e| e.to_string())?;
    Ok(u8::from_le_bytes(buff))
}

#[allow(dead_code)]
/// Reads a C style string from the reader
pub(crate) fn read_string(reader: &mut std::io::BufReader<&File>) -> Result<String, String> {
    let mut buff = vec![];
    reader
        .read_until('\0' as u8, &mut buff)
        .map_err(|e| e.to_string())?;
    Ok(String::from_utf8(buff).map_err(|e| e.to_string())?)
}

#[allow(dead_code)]
/// Reads until a 2 byte pattern is read, returning the bytes read including the pattern
pub(crate) fn read_until_bytes<R: Read>(reader: &mut R, pattern: [u8; 2]) -> io::Result<Vec<u8>> {
    let mut buff: Vec<u8> = Vec::new();
    let mut window = [0_u8; 2];
    let mut window_pos = 0;
    let mut byte = [0_u8; 1];

    // fill starting window
    for i in 0..2 {
        match reader.read_exact(&mut byte) {
            Ok(()) => {
                window[i] = byte[0];
                buff.push(byte[0]);
            }
            Err(e) => {
                return Err(e);
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
