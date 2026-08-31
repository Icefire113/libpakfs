//! The public read-only runtime API: map-like access to a pak file.

use std::{collections::HashMap, fs::File, path::Path};

use crate::serialization::{
    deserializer::read_structure,
    errors::PakError,
    pakfile::{Codec, MetaEntry, MetaKey},
};

/// An entry in the pak's cached manifest: the index into the parsed
/// manifest plus the decoded codec for that entry.
#[derive(Debug, Clone, Copy)]
struct Entry {
    manifest_index: usize,
    codec: Codec,
}

/// A read-only pak file. Map-like semantics: give a path, get bytes.
///
/// Opened paks are frozen; there are no mutable operations.
///
/// Only the header, manifest, and metadata are held in memory; file contents
/// are read from disk on demand using positioned reads, which do not disturb
/// any shared cursor and are safe to issue concurrently from multiple threads.
#[derive(Debug)]
pub struct PakFile {
    structure: crate::serialization::deserializer::PakStructure,
    file: File,
    /// path -> entry cache built once at open
    entries: HashMap<String, Entry>,
}

impl PakFile {
    /// Opens a pak file, parsing the header, manifest, and metadata, and
    /// caching the path -> entry table in memory. The data region is left on
    /// disk and read on demand.
    pub fn open(path: impl AsRef<Path>) -> Result<PakFile, PakError> {
        let file = File::options()
            .read(true)
            .write(false)
            .open(path.as_ref())?;
        let structure = read_structure(&file)?;

        let mut entries = HashMap::with_capacity(structure.manifest.len());
        for (i, m) in structure.manifest.iter().enumerate() {
            let codec = Codec::from_id(m.codec).ok_or(PakError::UnknownCodec(m.codec))?;
            entries.insert(
                m.path.clone(),
                Entry {
                    manifest_index: i,
                    codec,
                },
            );
        }

        Ok(PakFile {
            structure,
            file,
            entries,
        })
    }

    /// True if `path` exists in the pak.
    pub fn exists(&self, path: &str) -> bool {
        self.entries.contains_key(path)
    }

    /// Number of entries in the pak.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True if the pak has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the full, decompressed contents of `path`.
    pub fn get(&self, path: &str) -> Result<Vec<u8>, PakError> {
        let entry = self
            .entries
            .get(path)
            .ok_or_else(|| PakError::NotFound(path.to_string()))?;
        let m = &self.structure.manifest[entry.manifest_index];
        let compressed = self.structure.read_blob(&self.file, m)?;
        entry.codec.decompress(&compressed, m.original_size)
    }

    /// Reads the full, decompressed contents of `path` into `buf`.
    ///
    /// Errors with `BufferTooSmall` if `buf` is smaller than the file;
    /// never truncates silently.
    pub fn read_into(&self, path: &str, buf: &mut [u8]) -> Result<(), PakError> {
        let needed = self.size(path)?;
        if (buf.len() as u64) < needed {
            return Err(PakError::BufferTooSmall {
                needed,
                got: buf.len(),
            });
        }
        let data = self.get(path)?;
        buf[..needed as usize].copy_from_slice(&data);
        Ok(())
    }

    /// Uncompressed size of `path` in bytes.
    pub fn size(&self, path: &str) -> Result<u64, PakError> {
        let entry = self
            .entries
            .get(path)
            .ok_or_else(|| PakError::NotFound(path.to_string()))?;
        Ok(self.structure.manifest[entry.manifest_index].original_size)
    }

    /// Pak-level metadata; unknown keys are filtered out.
    pub fn metadata(&self) -> Vec<(MetaKey, u64)> {
        self.structure
            .metadata
            .iter()
            .filter_map(|MetaEntry { key, value }| MetaKey::from_id(*key).map(|k| (k, *value)))
            .collect()
    }

    /// All paths in the pak, sorted.
    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.structure.manifest.iter().map(|m| m.path.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::pakfile::Codec;
    use crate::serialization::serializer::PakWriter;

    /// Writes a small pak to a unique temp file.
    fn write_test_pak(
        files: &[(&str, &[u8], Codec)],
        meta: Option<(MetaKey, u64)>,
        name: &str,
    ) -> std::path::PathBuf {
        let mut w = PakWriter::new();
        for (path, data, codec) in files {
            w.add_bytes(path, *data, *codec).unwrap();
        }
        if let Some((k, v)) = meta {
            w.set_metadata(k, v);
        }
        let bytes = w.into_bytes().unwrap();

        let dir = std::env::temp_dir().join("libpakfs_tests");
        std::fs::create_dir_all(&dir).unwrap();
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = dir.join(format!(
            "{name}-{}.pak",
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::write(&path, &bytes).unwrap();
        path
    }

    fn make_test_pak() -> PakFile {
        PakFile::open(write_test_pak(
            &[
                ("b.txt", b"hello" as &[u8], Codec::None),
                ("a.txt", b"world" as &[u8], Codec::Lz4(0)),
            ],
            Some((MetaKey::ModifiedAt, 1234)),
            "get",
        ))
        .unwrap()
    }

    #[test]
    fn get_roundtrips() {
        let pak = make_test_pak();
        assert_eq!(pak.get("a.txt").unwrap(), b"world".to_vec());
        assert_eq!(pak.get("b.txt").unwrap(), b"hello".to_vec());
    }

    #[test]
    fn missing_path_is_not_found() {
        let pak = make_test_pak();
        assert!(matches!(pak.get("nope.txt"), Err(PakError::NotFound(_))));
        assert!(!pak.exists("nope.txt"));
        assert!(pak.exists("a.txt"));
    }

    #[test]
    fn size_and_len() {
        let pak = make_test_pak();
        assert_eq!(pak.len(), 2);
        assert_eq!(pak.size("a.txt").unwrap(), 5);
        assert_eq!(pak.size("b.txt").unwrap(), 5);
    }

    #[test]
    fn read_into_works_and_errors_on_small_buffer() {
        let pak = make_test_pak();

        let mut buf = [0u8; 5];
        pak.read_into("b.txt", &mut buf).unwrap();
        assert_eq!(&buf, b"hello");

        let mut small = [0u8; 3];
        assert!(matches!(
            pak.read_into("b.txt", &mut small),
            Err(PakError::BufferTooSmall { needed: 5, got: 3 })
        ));
    }

    #[test]
    fn paths_are_sorted() {
        let pak = make_test_pak();
        let paths: Vec<&str> = pak.paths().collect();
        assert_eq!(paths, vec!["a.txt", "b.txt"]);
    }

    #[test]
    fn concurrent_gets_are_safe() {
        use std::sync::Arc;
        let pak = Arc::new(make_test_pak());
        let mut handles = Vec::new();
        for _ in 0..4 {
            let pak = Arc::clone(&pak);
            handles.push(std::thread::spawn(move || {
                for _ in 0..50 {
                    assert_eq!(pak.get("a.txt").unwrap(), b"world".to_vec());
                    assert_eq!(pak.get("b.txt").unwrap(), b"hello".to_vec());
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
    }

    #[test]
    fn metadata_filters_unknown_keys() {
        let path = write_test_pak(
            &[("x", b"y" as &[u8], Codec::None)],
            Some((MetaKey::ToolId, 7)),
            "meta",
        );
        let mut bytes = std::fs::read(&path).unwrap();
        // corrupt the metadata key to an unknown id: metadata starts at
        // meta_offset = HEADER_SIZE + manifest_size; manifest is 27 + 1 = 28
        let meta_offset = 40 + 28;
        bytes[meta_offset] = 200;
        std::fs::write(&path, &bytes).unwrap();

        let pak = PakFile::open(&path).unwrap();
        assert_eq!(pak.metadata(), vec![]);
    }
}
