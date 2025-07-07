use std::{fs::File, io::Read};

use crate::{
    pakfile::pakfile::{PAKFILE_MAGIC, PAKFILE_VERSION, PakFile},
    serialization::errors::{self, DeSerError},
    util::{read_u32, read_u64},
};

#[derive(Debug)]
pub struct PakFileDeserializer {
    pak_file: File,
}

impl PakFileDeserializer {
    pub fn new(pak_file: File) -> Self {
        PakFileDeserializer { pak_file: pak_file }
    }
}

impl PakFileDeserializer {
    pub fn deserialize(&mut self) -> Result<PakFile, DeSerError> {
        let mut reader = std::io::BufReader::new(&self.pak_file);
        let mut header_bytes = [0u8; 4];

        reader.read_exact(header_bytes.as_mut())?;

        println!(
            "read magic: [{}, {}, {}, {}]\
           \n       aka: [{}, {}, {}, {}]",
            header_bytes[0] as char,
            header_bytes[1] as char,
            header_bytes[2] as char,
            header_bytes[3] as char,
            header_bytes[0],
            header_bytes[1],
            header_bytes[2],
            header_bytes[3]
        );

        if header_bytes != PAKFILE_MAGIC {
            return Err(errors::DeSerError::InvalidFileMagic);
        }

        let ver: u32 = read_u32(&mut reader)?;
        println!("read version: {}", ver);

        if ver != PAKFILE_VERSION {
            return Err(errors::DeSerError::InvalidFileVersion { actual: ver });
        }

        // skip the ID portion
        // reader
        //     .seek_relative(size_of::<u64>() as i64)
        //     .map_err(|e| e.to_string())?;
        let id: u64 = read_u64(&mut reader)?;
        println!("read file ID: {}", id);

        let entry_count = read_u64(&mut reader)?;
        println!("read entry count: {}", entry_count);

        Ok(PakFile::default())
    }
}
