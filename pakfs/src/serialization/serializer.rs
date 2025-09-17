use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::Path,
};

use crate::{pakfile::pakfile::PakFile, serialization::errors::SerializerError};

#[derive(Debug)]
pub struct PakFileSerializer {
    pak_file: PakFile,
}

impl PakFileSerializer {
    pub fn new(pak_file: PakFile) -> Self {
        PakFileSerializer { pak_file: pak_file }
    }
}

impl PakFileSerializer {
    /// Saves the pak file to the specified path
    /// NOTE: THE FILE WILL BE OVERWRITTEN IF IT ALREADY EXISTS
    pub fn save_to(&self, path: &Path) -> Result<(), SerializerError> {
        if !path.is_file() && path.exists() {
            return Err(SerializerError::NotAFileError);
        }

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        file.lock()?;
        let mut writer = BufWriter::new(file);
        writer.write(self.pak_file.header.as_bytes().as_slice())?;
        writer.write(self.pak_file.data.as_slice())?;

        writer.flush()?;
        Ok(())
    }
}
