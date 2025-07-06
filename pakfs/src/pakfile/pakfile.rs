/// The magic for a pak file, this is used to validate the header and should be the first 4 bytes of a pak file
pub const PAKFILE_MAGIC: [u8; 4] = ['p' as u8, 'k' as u8, 'f' as u8, 's' as u8];

/// The current version of the pak file
pub const PAKFILE_VERSION: u32 = 1;

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
    /// Create a new pak file header, with all empty (0d out) data
    fn default() -> Self {
        PakFileHeader {
            magic: [0_u8; 4],
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
            magic: PAKFILE_MAGIC,
            version: PAKFILE_VERSION,
            id: rand::random(),
            entry_count: 0,
            manifest: vec![],
        }
    }

    /// Sets the pak file's ID
    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct PakFileData {
    /// The files in the pak file, stored as a b-tree
    data: Vec<u8>,
}

impl Default for PakFileData {
    fn default() -> Self {
        PakFileData { data: vec![] }
    }
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
#[derive(Debug, Default)]
pub struct PakFile {
    header: PakFileHeader,
    data: PakFileData,
}
