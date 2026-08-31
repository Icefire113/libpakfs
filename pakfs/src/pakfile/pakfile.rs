use std::{collections::HashMap, fs::File, path::PathBuf};

use crate::{pakfile::errors::PakFileError, serialization::deserializer::PakFileDeserializer};

#[derive(Default, Debug)]
struct PakFile {
    files: HashMap<String, Vec<u8>>,
}

impl PakFile {
    /// Returns the file if it exists or `None`
    pub fn get_file(&self, filename: &str) -> Option<Vec<u8>> {
        self.files.get(filename).cloned()
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<PakFile, PakFileError> {
        let mut pf = PakFile::default();
        let file = File::options().read(true).open(path.into())?;
        let mut pfd = PakFileDeserializer::new(file).deserialize()?;
        pfd.header.manifest().iter().for_each(|e| {
            let v = vec![];
            todo!("Copy bytes of length e.file_size starting at position pfd.data[e.file_offset]");
            pf.files.insert(e.file_path().to_string(), v);
        });

        Ok(pf)
    }
}
