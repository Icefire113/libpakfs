/// The magic for a pak file, this is used to validate the header and should be the first 4 bytes of a pak file
pub const PAKFILE_MAGIC: [u8; 4] = ['p' as u8, 'k' as u8, 'f' as u8, 's' as u8];

#[allow(dead_code)]
/// This is the format for a pak file header
#[derive(Debug)]
pub struct PakFileHeader {
    /// This should always be "pkfs" (not-null-terminated)
    magic: [u8; 4],
    /// Version of the pak file
    version: u32,
    /// The ID for the pak file, these should be unique for each pak file that you intend on loading
    id: u64,
    /// The number of files in the pak file
    entry_count: u64,
    /// List of all files in the pak file
    manifest: Vec<ManifestEntry>,
}

impl Default for PakFileHeader {
    fn default() -> Self {
        PakFileHeader {
            magic: PAKFILE_MAGIC,
            version: 1,
            id: 0,
            entry_count: 0,
            manifest: vec![],
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct PakFileData {
    /// The files in the pak file, stored as a b-tree
    data: [u8],
}

/// This represents the format for a manifest entry
#[allow(dead_code)]
#[derive(Debug)]
pub struct ManifestEntry {
    /// The path to the file localized to the pak file
    file_path: String,
    /// The offset of the file stored in the pak file indexed from the start of the data portion
    file_offset: u64,
    /// The size of the file in bytes
    file_size: u64,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct PakFile {
    header: PakFileHeader,
    data: PakFileData,
}
