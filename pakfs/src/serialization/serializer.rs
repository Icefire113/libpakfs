//! Pak file writing: staging entries and writing the on-disk format.
//!
//! The low-level writer is [`PakWriter`]; the public build API (`PakBuilder`)
//! will sit on top of this.

use std::{collections::BTreeMap, io::Read, path::Path};

use crate::serialization::errors::PakError;
use crate::serialization::pakfile::{
    Codec, HEADER_SIZE, Header, ManifestEntry, MetaEntry, MetaKey,
};

/// A staged file: its contents (already read) and per-entry codec.
#[derive(Debug)]
struct StagedFile {
    data: Vec<u8>,
    codec: Codec,
}

/// Stages files and pak-level metadata, then writes a complete pak file.
///
/// A saved pak is frozen: it is never modified after `save`.
#[derive(Debug, Default)]
pub struct PakWriter {
    /// Staged files keyed by path. `BTreeMap` for deterministic, sorted output.
    files: BTreeMap<String, StagedFile>,
    /// Pak-level metadata, keyed by on-disk key id.
    metadata: BTreeMap<u16, u64>,
}

impl PakWriter {
    /// Creates an empty pak writer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Stages a file, streaming from `src` (read fully into memory for now;
    /// a streaming variant is planned for the future).
    /// `codec` selects the compression applied to this entry only.
    pub fn add_file(
        &mut self,
        path: &str,
        mut src: impl Read,
        codec: Codec,
    ) -> Result<&mut Self, PakError> {
        if self.files.contains_key(path) {
            return Err(PakError::DuplicatePath(path.to_string()));
        }
        if path.is_empty() {
            return Err(PakError::Malformed("path must not be empty"));
        }
        let mut data = Vec::new();
        src.read_to_end(&mut data)?;
        self.files
            .insert(path.to_string(), StagedFile { data, codec });
        Ok(self)
    }

    /// Stages bytes directly.
    pub fn add_bytes(
        &mut self,
        path: &str,
        bytes: &[u8],
        codec: Codec,
    ) -> Result<&mut Self, PakError> {
        if self.files.contains_key(path) {
            return Err(PakError::DuplicatePath(path.to_string()));
        }
        if path.is_empty() {
            return Err(PakError::Malformed("path must not be empty"));
        }
        self.files.insert(
            path.to_string(),
            StagedFile {
                data: bytes.to_vec(),
                codec,
            },
        );
        Ok(self)
    }

    /// Sets pak-level metadata. Later sets of the same key overwrite.
    pub fn set_metadata(&mut self, key: MetaKey, value: u64) -> &mut Self {
        self.metadata.insert(key.id(), value);
        self
    }

    /// Consumes the writer and produces the complete on-disk pak file as bytes.
    ///
    /// The output is deterministic: the manifest is sorted by path (guaranteed
    /// by the `BTreeMap`), and all fields are written in manifest order.
    pub fn into_bytes(self) -> Result<Vec<u8>, PakError> {
        // Compress every staged file, building the manifest as we go.
        let mut entries: Vec<ManifestEntry> = Vec::with_capacity(self.files.len());
        let mut data: Vec<u8> = Vec::new();
        for (path, staged) in &self.files {
            if path.len() > u16::MAX as usize {
                return Err(PakError::Malformed("path longer than u16::MAX bytes"));
            }
            let compressed = staged.codec.compress(&staged.data)?;
            let offset = data.len() as u64;
            data.extend_from_slice(&compressed);
            entries.push(ManifestEntry {
                offset,
                compressed_size: compressed.len() as u64,
                original_size: staged.data.len() as u64,
                codec: staged.codec.id(),
                path: path.clone(),
            });
        }

        let manifest: Vec<u8> = entries.iter().flat_map(|e| e.to_bytes()).collect();
        let manifest_size: u16 = manifest
            .len()
            .try_into()
            .map_err(|_| PakError::Malformed("manifest larger than u16::MAX bytes"))?;

        let meta: Vec<u8> = self
            .metadata
            .iter()
            .flat_map(|(&key, &value)| MetaEntry { key, value }.to_bytes())
            .collect();

        // Layout: header | manifest | metadata | data
        let header = Header {
            magic: crate::serialization::pakfile::PAKFILE_MAGIC,
            entry_count: entries.len() as u64,
            data_offset: HEADER_SIZE + manifest.len() as u64 + meta.len() as u64,
            meta_offset: HEADER_SIZE + manifest.len() as u64,
            meta_count: self.metadata.len() as u64,
            manifest_size,
            reserved: 0,
        };

        let mut out =
            Vec::with_capacity(HEADER_SIZE as usize + manifest.len() + meta.len() + data.len());
        out.extend_from_slice(&header.to_bytes());
        out.extend_from_slice(&manifest);
        out.extend_from_slice(&meta);
        out.extend_from_slice(&data);
        Ok(out)
    }

    /// Consumes the writer, writing the pak file to `out`.
    /// The file will be overwritten if it already exists.
    pub fn save(self, out: impl AsRef<Path>) -> Result<(), PakError> {
        let bytes = self.into_bytes()?;
        std::fs::write(out.as_ref(), bytes)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::pakfile::PAKFILE_MAGIC;

    /// Writes a small pak and verifies the raw bytes match the spec layout.
    #[test]
    fn written_bytes_match_spec() {
        let mut w = PakWriter::new();
        w.add_bytes("b.txt", b"hello", Codec::None).unwrap();
        w.add_bytes("a.txt", b"world", Codec::Lz4(0)).unwrap();
        w.set_metadata(MetaKey::ModifiedAt, 1234);
        let bytes = w.into_bytes().unwrap();

        // header
        let header = Header::from_bytes(&bytes[0..HEADER_SIZE as usize]).unwrap();
        assert_eq!(header.magic, PAKFILE_MAGIC);
        assert_eq!(header.entry_count, 2);
        assert_eq!(header.meta_count, 1);
        // 2 entries: 27 fixed bytes + 5 byte path each
        assert_eq!(header.manifest_size, 64);
        assert_eq!(header.meta_offset, HEADER_SIZE + 64);
        assert_eq!(header.data_offset, header.meta_offset + 10);

        // manifest, sorted by path: a.txt then b.txt
        let manifest = &bytes[HEADER_SIZE as usize..header.meta_offset as usize];
        let second_start = 32;
        let first = ManifestEntry {
            offset: 0,
            compressed_size: u64::from_le_bytes(manifest[8..16].try_into().unwrap()),
            original_size: 5,
            codec: 2,
            path: "a.txt".into(),
        };
        assert_eq!(&manifest[0..second_start], &first.to_bytes()[..]);

        let second = ManifestEntry {
            offset: first.compressed_size,
            compressed_size: 5,
            original_size: 5,
            codec: 0,
            path: "b.txt".into(),
        };
        assert_eq!(&manifest[second_start..], &second.to_bytes()[..]);

        // metadata: key 0 (ModifiedAt), value 1234
        let meta = &bytes[header.meta_offset as usize..header.data_offset as usize];
        assert_eq!(
            meta,
            &MetaEntry {
                key: 0,
                value: 1234
            }
            .to_bytes()
        );

        // data region: lz4("world") followed by "hello"
        let data = &bytes[header.data_offset as usize..];
        assert_eq!(data.len() as u64, first.compressed_size + 5);
        assert_eq!(&data[first.compressed_size as usize..], b"hello");
        assert_eq!(
            Codec::Lz4(0)
                .decompress(&data[0..first.compressed_size as usize], 5)
                .unwrap(),
            b"world".to_vec()
        );
    }

    #[test]
    fn duplicate_paths_rejected() {
        let mut w = PakWriter::new();
        w.add_bytes("a", b"1", Codec::None).unwrap();
        assert!(matches!(
            w.add_bytes("a", b"2", Codec::None),
            Err(PakError::DuplicatePath(_))
        ));
    }

    #[test]
    fn empty_paths_rejected() {
        let mut w = PakWriter::new();
        assert!(matches!(
            w.add_bytes("", b"1", Codec::None),
            Err(PakError::Malformed(_))
        ));
    }
}
