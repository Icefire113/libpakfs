/// The magic for a pak file, this is used to validate the header and should be the first 4 bytes of a pak file
pub const PAKFILE_MAGIC: [u8; 4] = ['p' as u8, 'k' as u8, 'f' as u8, 's' as u8];

/// The current version of the pak file
pub const PAKFILE_VERSION: u32 = 1;

#[allow(dead_code)]
#[derive(Debug)]
/// This is the format for a pak file header
pub struct PakFileHeader {
    /// This should always be "pkfs" (not null-terminated)
    magic: [u8; 4],
    /// Version of the pak file
    version: u32,
    /// The ID for the pak file
    id: u64,
    /// The number of files in the pak file (also the number of manifest entries)
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

    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }

    pub fn set_magic(&mut self, magic: [u8; 4]) {
        self.magic = magic;
    }

    pub fn set_entry_count(&mut self, entry_count: u64) {
        self.entry_count = entry_count;
    }

    pub fn set_version(&mut self, version: u32) {
        self.version = version;
    }

    pub fn manifest_mut(&mut self) -> &mut Vec<ManifestEntry> {
        &mut self.manifest
    }

    pub fn magic(&self) -> [u8; 4] {
        self.magic
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn entry_count(&self) -> u64 {
        self.entry_count
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct PakFileData {
    /// The files in the pak file, stored as a b-tree
    data: Vec<u8>,
}

impl PakFileData {
    pub fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }
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
}

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct PakFile {
    header: PakFileHeader,
    data: PakFileData,
}
