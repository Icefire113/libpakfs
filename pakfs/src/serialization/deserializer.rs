use std::{fs::File, io::BufReader};

use crate::{
    pakfile::pakfile::{PAKFILE_MAGIC, PAKFILE_VERSION, PakFile},
    serialization::errors::{self, DeSerError},
    util::{read_n_bytes, read_u32, read_u64},
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
        let mut reader = BufReader::new(&self.pak_file);
        let magic_bytes = read_n_bytes(&mut reader, 4)?;

        println!(
            "read magic: [{}, {}, {}, {}]\
           \n       aka: [{}, {}, {}, {}]",
            magic_bytes[0] as char,
            magic_bytes[1] as char,
            magic_bytes[2] as char,
            magic_bytes[3] as char,
            magic_bytes[0],
            magic_bytes[1],
            magic_bytes[2],
            magic_bytes[3]
        );

        if magic_bytes != PAKFILE_MAGIC {
            return Err(errors::DeSerError::InvalidFileMagic);
        }

        let ver = read_u32(&mut reader)?;
        println!("read version: {}", ver);

        if ver != PAKFILE_VERSION {
            return Err(errors::DeSerError::InvalidFileVersion { actual: ver });
        }

        let id = read_u64(&mut reader)?;
        println!("read file ID: {}", id);

        let entry_count = read_u64(&mut reader)?;
        println!("read entry count: {}", entry_count);

        Ok(PakFile::default())
    }
}
