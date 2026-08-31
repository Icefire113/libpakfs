use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::Path,
};

use crate::{serialization::errors::SerializerError, serialization::pakfile::PakFileData};

#[derive(Debug)]
pub struct PakFileSerializer {
    pak_file: PakFileData,
}

impl PakFileSerializer {
    pub fn new(pak_file: PakFileData) -> Self {
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
            .truncate(true)
            .open(path)?;
        file.lock()?;

        let mut writer = BufWriter::new(&file);
        writer.write(self.pak_file.header.as_bytes().as_slice())?;
        writer.write(self.pak_file.data.as_slice())?;

        writer.flush()?;
        file.unlock()?;
        Ok(())
    }
}
