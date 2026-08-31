//! On-disk format types for pak files.
//!
//! See `docs/pakfile.md` for the full format specification.
//! All integers are little-endian on disk.

use std::io::{Read, Write};

use crate::serialization::errors::PakError;

/// The magic for a pak file, must be the first 4 bytes of the file.
pub const PAKFILE_MAGIC: [u8; 4] = *b"pkfs";

/// Total size of the fixed header in bytes.
pub const HEADER_SIZE: u64 = 40;

/// Per-entry compression codec, chosen at build time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    /// Stored raw
    None,
    /// Zstd frame format; level is a build-time choice and is not stored
    Zstd(u8),
    /// LZ4 frame format; level is a build-time choice and is not stored
    Lz4(u8),
}

impl Codec {
    /// The on-disk codec id byte.
    pub fn id(self) -> u8 {
        match self {
            Codec::None => 0,
            Codec::Zstd(_) => 1,
            Codec::Lz4(_) => 2,
        }
    }

    /// Maps an on-disk codec id to a codec. `None` for unknown ids.
    pub fn from_id(id: u8) -> Option<Codec> {
        match id {
            0 => Some(Codec::None),
            1 => Some(Codec::Zstd(0)),
            2 => Some(Codec::Lz4(0)),
            _ => None,
        }
    }

    /// Compresses `data` with this codec. `Codec::None` returns the input as-is.
    pub(crate) fn compress(self, data: &[u8]) -> Result<Vec<u8>, PakError> {
        match self {
            Codec::None => Ok(data.to_vec()),
            Codec::Zstd(level) => {
                let mut out = Vec::new();
                let mut enc = zstd::stream::Encoder::new(&mut out, level as i32)?;
                enc.write_all(data)?;
                enc.finish()?;
                Ok(out)
            }
            Codec::Lz4(level) => {
                let mut out = Vec::new();
                let mut enc = lz4::EncoderBuilder::new()
                    .level(level as u32)
                    .build(&mut out)?;
                enc.write_all(data)?;
                let (_w, res) = enc.finish();
                res?;
                Ok(out)
            }
        }
    }

    /// Decompresses `data` with this codec. `Codec::None` returns the input as-is.
    pub(crate) fn decompress(self, data: &[u8], original_size: u64) -> Result<Vec<u8>, PakError> {
        match self {
            Codec::None => Ok(data.to_vec()),
            Codec::Zstd(_) => {
                let mut out = Vec::with_capacity(original_size as usize);
                zstd::stream::Decoder::new(data)?.read_to_end(&mut out)?;
                Ok(out)
            }
            Codec::Lz4(_) => {
                let mut out = Vec::with_capacity(original_size as usize);
                let mut dec = lz4::Decoder::new(data)?;
                dec.read_to_end(&mut out)?;
                Ok(out)
            }
        }
    }
}

/// Typed pak-level metadata keys. Stored as `u16` ids on disk with `u64` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetaKey {
    /// Unix timestamp (seconds) of pak creation
    ModifiedAt,
    /// Build-tool defined identifier
    ToolId,
}

impl MetaKey {
    /// The on-disk metadata key id.
    pub fn id(self) -> u16 {
        match self {
            MetaKey::ModifiedAt => 0,
            MetaKey::ToolId => 1,
        }
    }

    /// Maps an on-disk metadata key id to a key. `None` for unknown keys
    /// (readers must ignore unknown keys).
    pub fn from_id(id: u16) -> Option<MetaKey> {
        match id {
            0 => Some(MetaKey::ModifiedAt),
            1 => Some(MetaKey::ToolId),
            _ => None,
        }
    }
}

/// The fixed 40-byte header at offset 0 of a pak file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    /// Must be `pkfs`
    pub magic: [u8; 4],
    /// Number of manifest entries
    pub entry_count: u64,
    /// Byte offset from start of file to the data region
    pub data_offset: u64,
    /// Byte offset from start of file to the metadata block
    pub meta_offset: u64,
    /// Number of pak-level metadata entries
    pub meta_count: u64,
    /// Total byte size of the manifest section
    pub manifest_size: u16,
    /// Reserved, must be 0
    pub reserved: u16,
}

impl Header {
    /// Serializes the header to its on-disk representation.
    pub fn to_bytes(&self) -> [u8; HEADER_SIZE as usize] {
        let mut out = [0u8; HEADER_SIZE as usize];
        out[0..4].copy_from_slice(&self.magic);
        out[4..12].copy_from_slice(&self.entry_count.to_le_bytes());
        out[12..20].copy_from_slice(&self.data_offset.to_le_bytes());
        out[20..28].copy_from_slice(&self.meta_offset.to_le_bytes());
        out[28..36].copy_from_slice(&self.meta_count.to_le_bytes());
        out[36..38].copy_from_slice(&self.manifest_size.to_le_bytes());
        out[38..40].copy_from_slice(&self.reserved.to_le_bytes());
        out
    }

    /// Parses a header from exactly `HEADER_SIZE` bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Header, PakError> {
        if bytes.len() != HEADER_SIZE as usize {
            return Err(PakError::Malformed("header must be 40 bytes"));
        }
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&bytes[0..4]);
        if magic != PAKFILE_MAGIC {
            return Err(PakError::BadMagic);
        }
        Ok(Header {
            magic,
            entry_count: u64::from_le_bytes(bytes[4..12].try_into().unwrap()),
            data_offset: u64::from_le_bytes(bytes[12..20].try_into().unwrap()),
            meta_offset: u64::from_le_bytes(bytes[20..28].try_into().unwrap()),
            meta_count: u64::from_le_bytes(bytes[28..36].try_into().unwrap()),
            manifest_size: u16::from_le_bytes(bytes[36..38].try_into().unwrap()),
            reserved: u16::from_le_bytes(bytes[38..40].try_into().unwrap()),
        })
    }
}

/// One manifest entry describing a file stored in the data region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestEntry {
    /// Byte offset of the file's data relative to `data_offset`
    pub offset: u64,
    /// Size of the stored (possibly compressed) blob in bytes
    pub compressed_size: u64,
    /// Size of the file after decompression
    pub original_size: u64,
    /// Compression codec id
    pub codec: u8,
    /// UTF-8 path, no trailing null
    pub path: String,
}

impl ManifestEntry {
    /// Serializes this entry to its on-disk representation.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(25 + self.path.len());
        out.extend_from_slice(&self.offset.to_le_bytes());
        out.extend_from_slice(&self.compressed_size.to_le_bytes());
        out.extend_from_slice(&self.original_size.to_le_bytes());
        out.push(self.codec);
        // path length is u16; a path longer than u16::MAX is rejected at build time
        out.extend_from_slice(&(self.path.len() as u16).to_le_bytes());
        out.extend_from_slice(self.path.as_bytes());
        out
    }

    /// The on-disk size of this entry in bytes.
    pub fn disk_size(&self) -> u64 {
        27 + self.path.len() as u64
    }
}

/// One pak-level metadata key/value pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetaEntry {
    /// Metadata key id
    pub key: u16,
    /// Value; meaning depends on the key
    pub value: u64,
}

impl MetaEntry {
    /// Serializes this entry to its on-disk representation.
    pub fn to_bytes(&self) -> [u8; 10] {
        let mut out = [0u8; 10];
        out[0..2].copy_from_slice(&self.key.to_le_bytes());
        out[2..10].copy_from_slice(&self.value.to_le_bytes());
        out
    }
}
