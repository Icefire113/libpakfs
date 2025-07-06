use std::path::Path;

use crate::pakfile::pakfile::PakFile;

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
    pub fn save_to(&self, path: &Path) -> Result<(), String> {
        Ok(())
    }
}
