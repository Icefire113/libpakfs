use std::path::Path;

use crate::serialization::errors::FileSaveError;

/// The magic for a pak file, this is used to validate the header and should be the first 4 bytes of a pak file
pub const PAKFILE_MAGIC: &[u8; 4] = b"pkfs";

/// The current version of the pak file
pub const PAKFILE_VERSION: u32 = 1;

#[allow(dead_code)]
#[derive(Debug)]
/// This is the format for a pak file header
pub struct PakFileHeader {
    /// This should always be "pkfs" (not null-terminated)
    pub magic: [u8; 4],
    /// Version of the pak file
    pub version: u32,
    /// The ID for the pak file
    pub id: u64,
    /// The number of files in the pak file (also the number of manifest entries)
    pub entry_count: u64,
    /// List of all files in the pak file
    manifest: Vec<ManifestEntry>,
}

impl Default for PakFileHeader {
    /// Create a new pak file header, with all empty (0d out) data
    fn default() -> Self {
        PakFileHeader {
            magic: [0u8; 4],
            version: 0,
            id: 0,
            entry_count: 0,
            manifest: vec![],
        }
    }
}

impl PakFileHeader {
    /// Creates a new pak file header with expected file magic, current libpakfs
    /// version and a random ID
    pub fn new() -> Self {
        PakFileHeader {
            magic: *PAKFILE_MAGIC,
            version: PAKFILE_VERSION,
            id: rand::random(),
            entry_count: 0,
            manifest: vec![],
        }
    }

    /// Creates and returns a vector of bytes representing the header
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut v: Vec<u8> = Vec::new();
        v.extend_from_slice(&self.magic);
        v.extend_from_slice(&self.version.to_le_bytes());
        v.extend_from_slice(&self.id.to_le_bytes());
        v.extend_from_slice(&self.entry_count.to_le_bytes());

        // deep copy the manifest
        self.manifest.iter().enumerate().for_each(|(_, item)| {
            v.extend_from_slice(&item.file_offset.to_le_bytes());
            v.extend_from_slice(&item.file_size.to_le_bytes());
            v.extend_from_slice(&item.file_path.as_bytes());
            // stupid c-string null terminator
            v.extend_from_slice(&[0u8]);
        });

        v
    }

    pub fn manifest_mut(&mut self) -> &mut Vec<ManifestEntry> {
        &mut self.manifest
    }

    pub fn manifest(&self) -> &Vec<ManifestEntry> {
        &self.manifest
    }
}

/// This represents the format for a manifest entry
#[allow(dead_code)]
#[derive(Debug)]
pub struct ManifestEntry {
    /// The offset of the file stored in the pak file indexed from the start of the data portion
    file_offset: u64,
    /// The size of the file in bytes
    file_size: u64,
    /// The path to the file localized to the pak file
    file_path: String,
}

impl ManifestEntry {
    /// Creates a new manifest entry with the given file path, offset and size
    pub fn new(file_path: String, file_offset: u64, file_size: u64) -> Self {
        ManifestEntry {
            file_path,
            file_offset,
            file_size,
        }
    }

    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    pub fn file_offset(&self) -> u64 {
        self.file_offset
    }
}

/// Struct representing a pak file as it would appear saved on disk
#[allow(dead_code)]
#[derive(Debug)]
pub struct PakFileData {
    pub header: PakFileHeader,
    pub data: Vec<u8>,
}

impl Default for PakFileData {
    /// Creates a new pak file with 0d out header and an empty data portion.
    fn default() -> Self {
        PakFileData {
            header: PakFileHeader::default(),
            data: vec![],
        }
    }
}

impl PakFileData {
    /// Creates a new pak file, with a correctly formatted header and an empty data portion
    pub fn new() -> Self {
        PakFileData {
            header: PakFileHeader::new(),
            data: vec![],
        }
    }

    /// Adds a file to the pak file, note that this will read the entire file into memory
    /// a variant of this function that does not read the entire file into memory is
    /// planned for the future
    pub fn add_file(&mut self, file_path: &Path) -> Result<(), FileSaveError> {
        Ok(())
    }
}
