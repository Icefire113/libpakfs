use std::{fs::File, io::BufReader};

use crate::{
    pakfile::pakfile::{ManifestEntry, PAKFILE_MAGIC, PAKFILE_VERSION, PakFile, PakFileHeader},
    serialization::errors::DeSerError,
    util::{read_n_bytes, read_string, read_u32, read_u64},
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
            return Err(DeSerError::InvalidFileMagic);
        }

        let ver = read_u32(&mut reader)?;
        println!("read version: {}", ver);

        if ver != PAKFILE_VERSION {
            return Err(DeSerError::InvalidFileVersion { actual: ver });
        }

        let id = read_u64(&mut reader)?;
        println!("read file ID: {}", id);

        let entry_count = read_u64(&mut reader)?;
        println!("read entry count: {}", entry_count);

        // At this point, we have read most of the header with the exception of the
        // manifest and file data.
        let mut header = PakFileHeader::default();
        header.set_id(id);
        header.set_entry_count(entry_count);
        header.set_magic([
            magic_bytes[0],
            magic_bytes[1],
            magic_bytes[2],
            magic_bytes[3],
        ]);
        header.set_version(ver);

        let mut data_size = 0;

        for i in 0..entry_count {
            let file_offset = read_u64(&mut reader)?;
            let file_size = read_u64(&mut reader)?;
            let file_path = read_string(&mut reader)?;

            let entry = ManifestEntry::new(file_path, file_offset, file_size);
            println!("read entry {}: {:#?}", i, entry);
            header.manifest_mut().push(entry);
            data_size += file_size;
        }

        println!("\nread header: {:#?}", header);

        let mut pak_file = PakFile::default();
        pak_file.set_header(header);

        let file_data = read_n_bytes(&mut reader, data_size as usize)?;
        pak_file.set_data(file_data);
        
        println!("\nread pak file: {:#?}", pak_file);

        Ok(pak_file)
    }
}
