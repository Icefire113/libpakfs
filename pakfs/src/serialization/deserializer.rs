use std::{fs::File, io::Read};

use crate::pakfile::pakfile::{PAKFILE_MAGIC, PakFile};

#[derive(Debug)]
pub struct PakFileDeSerializer {
    pak_file: File,
}

impl PakFileDeSerializer {
    pub fn new(pak_file: File) -> Self {
        PakFileDeSerializer { pak_file: pak_file }
    }
}

impl PakFileDeSerializer {
    pub fn deserialize(&mut self) -> Result<PakFile, String> {
        let mut reader = std::io::BufReader::new(&self.pak_file);
        let mut header_bytes = [0u8; 4];

        reader
            .read_exact(header_bytes.as_mut())
            .map_err(|e| e.to_string())?;

        println!(
            "read magic: [{}, {}, {}, {}]",
            header_bytes[0] as char,
            header_bytes[1] as char,
            header_bytes[2] as char,
            header_bytes[3] as char
        );

        if header_bytes != PAKFILE_MAGIC {
            return Err("invalid magic".to_string());
        }

        let ver: u32 = read_u32(&mut reader)?;
        println!("read version: {}", ver);

        // skip the ID portion
        reader
            .seek_relative(size_of::<u64>() as i64)
            .map_err(|e| e.to_string())?;

        let entry_count = read_u64(&mut reader)?;
        println!("read entry count: {}", entry_count);

        Ok(PakFile::default())
    }
}

/// Reads a u32 from the reader
fn read_u32(reader: &mut std::io::BufReader<&File>) -> Result<u32, String> {
    let mut buff = [0u8; size_of::<u32>()];
    reader.read_exact(&mut buff).map_err(|e| e.to_string())?;
    Ok(u32::from_le_bytes(buff))
}

/// Reads a u64 from the reader
fn read_u64(reader: &mut std::io::BufReader<&File>) -> Result<u64, String> {
    let mut buff = [0u8; size_of::<u64>()];
    reader.read_exact(&mut buff).map_err(|e| e.to_string())?;
    Ok(u64::from_le_bytes(buff))
}
