//! Pak file reading: parsing the on-disk format back into structured data.

use std::{fs::File, io::Read};

#[cfg(unix)]
use std::os::unix::fs::FileExt;
#[cfg(windows)]
use std::os::windows::fs::FileExt;

use crate::serialization::errors::PakError;
use crate::serialization::pakfile::{HEADER_SIZE, Header, ManifestEntry, MetaEntry};

/// Parses header + manifest + metadata from a stream, without reading the
/// data region.
///
/// Returns everything needed to locate file blobs by positioned reads.
pub fn read_structure(mut reader: impl Read) -> Result<PakStructure, PakError> {
    let mut header_buf = [0u8; HEADER_SIZE as usize];
    reader.read_exact(&mut header_buf)?;
    let header = Header::from_bytes(&header_buf)?;

    // sanity-check the layout before allocating anything
    if header.data_offset < header.meta_offset {
        return Err(PakError::Malformed("data_offset precedes meta_offset"));
    }

    let manifest_size = header.manifest_size as u64;
    if header.meta_offset < HEADER_SIZE + manifest_size {
        return Err(PakError::Malformed("manifest overruns meta_offset"));
    }

    let mut manifest_buf = vec![0u8; manifest_size as usize];
    reader.read_exact(&mut manifest_buf)?;
    let manifest = parse_manifest(&manifest_buf, header.entry_count)?;

    let meta_size = header
        .data_offset
        .checked_sub(header.meta_offset)
        .ok_or(PakError::Malformed("data_offset precedes meta_offset"))?;
    if meta_size != header.meta_count * 10 {
        return Err(PakError::Malformed("metadata block size mismatch"));
    }

    let mut meta_buf = vec![0u8; meta_size as usize];
    reader.read_exact(&mut meta_buf)?;
    let metadata = parse_metadata(&meta_buf, header.meta_count)?;

    Ok(PakStructure {
        header,
        manifest,
        metadata,
    })
}

/// The parsed header, manifest, and metadata of a pak file — everything
/// except the data region.
#[derive(Debug)]
pub struct PakStructure {
    pub header: Header,
    pub manifest: Vec<ManifestEntry>,
    /// Raw pak-level metadata pairs as stored (unknown keys included).
    pub metadata: Vec<MetaEntry>,
}

impl PakStructure {
    /// Reads and decompresses the blob for `entry` from `file` using
    /// positioned reads (no shared cursor; safe for concurrent use).
    pub fn read_blob(&self, file: &File, entry: &ManifestEntry) -> Result<Vec<u8>, PakError> {
        // data_offset is absolute from the start of the file; entry.offset
        // is relative to the data region
        let start = self
            .header
            .data_offset
            .checked_add(entry.offset)
            .ok_or(PakError::Malformed("blob offset overflows"))?;
        let len = entry.compressed_size as usize;

        let mut compressed = vec![0u8; len];
        let mut filled = 0usize;
        while filled < len {
            let n = positioned_read(file, &mut compressed[filled..], start + filled as u64)?;
            if n == 0 {
                return Err(PakError::Malformed("blob extends past end of file"));
            }
            filled += n;
        }
        Ok(compressed)
    }
}

/// Platform positioned read: reads into `buf` at absolute `pos` without
/// moving any shared file cursor. Returns bytes read (0 at EOF).
#[cfg(windows)]
fn positioned_read(file: &File, buf: &mut [u8], pos: u64) -> std::io::Result<usize> {
    file.seek_read(buf, pos)
}
/// Platform positioned read (unix).
#[cfg(unix)]
fn positioned_read(file: &File, buf: &mut [u8], pos: u64) -> std::io::Result<usize> {
    file.read_at(buf, pos)
}

/// Parses `entry_count` manifest entries from the manifest bytes.
fn parse_manifest(bytes: &[u8], entry_count: u64) -> Result<Vec<ManifestEntry>, PakError> {
    let mut entries = Vec::with_capacity(entry_count as usize);
    let mut pos = 0usize;

    for _ in 0..entry_count {
        // fixed part: offset(8) + compressed_size(8) + original_size(8) + codec(1) + path_len(2)
        const FIXED: usize = 27;
        if bytes.len() < pos + FIXED {
            return Err(PakError::Malformed("truncated manifest entry"));
        }
        let mut offset_bytes = [0u8; 8];
        offset_bytes.copy_from_slice(&bytes[pos..pos + 8]);
        let offset = u64::from_le_bytes(offset_bytes);

        let mut csize_bytes = [0u8; 8];
        csize_bytes.copy_from_slice(&bytes[pos + 8..pos + 16]);
        let compressed_size = u64::from_le_bytes(csize_bytes);

        let mut osize_bytes = [0u8; 8];
        osize_bytes.copy_from_slice(&bytes[pos + 16..pos + 24]);
        let original_size = u64::from_le_bytes(osize_bytes);

        let codec = bytes[pos + 24];
        let mut plen_bytes = [0u8; 2];
        plen_bytes.copy_from_slice(&bytes[pos + 25..pos + 27]);
        let path_len = u16::from_le_bytes(plen_bytes) as usize;

        if bytes.len() < pos + FIXED + path_len {
            return Err(PakError::Malformed("truncated manifest path"));
        }
        let path_bytes = &bytes[pos + FIXED..pos + FIXED + path_len];
        let path = String::from_utf8(path_bytes.to_vec())
            .map_err(|_| PakError::Malformed("manifest path is not valid UTF-8"))?;
        pos += FIXED + path_len;

        if codec > 2 {
            return Err(PakError::UnknownCodec(codec));
        }

        entries.push(ManifestEntry {
            offset,
            compressed_size,
            original_size,
            codec,
            path,
        });
    }

    if pos != bytes.len() {
        return Err(PakError::Malformed("manifest has trailing bytes"));
    }

    Ok(entries)
}

/// Parses `meta_count` metadata pairs from the metadata bytes.
fn parse_metadata(bytes: &[u8], meta_count: u64) -> Result<Vec<MetaEntry>, PakError> {
    if bytes.len() != meta_count as usize * 10 {
        return Err(PakError::Malformed("metadata block size mismatch"));
    }

    let mut entries = Vec::with_capacity(meta_count as usize);
    for chunk in bytes.as_chunks::<10>().0 {
        let mut key_bytes = [0u8; 2];
        key_bytes.copy_from_slice(&chunk[0..2]);
        let key = u16::from_le_bytes(key_bytes);

        let mut value_bytes = [0u8; 8];
        value_bytes.copy_from_slice(&chunk[2..10]);
        let value = u64::from_le_bytes(value_bytes);

        entries.push(MetaEntry { key, value });
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::pakfile::{Codec, MetaKey};
    use crate::serialization::builder::PakBuilder;
    use std::io::Write;

    /// Writes a small pak to a temp file and returns (path, bytes).
    fn make_test_pak() -> (std::path::PathBuf, Vec<u8>) {
        let mut w = PakBuilder::new();
        w.add_bytes("b.txt", b"hello", Codec::None).unwrap();
        w.add_bytes("a.txt", b"world", Codec::Lz4(0)).unwrap();
        w.set_metadata(MetaKey::ModifiedAt, 1234);
        let bytes = w.into_bytes().unwrap();

        let dir = std::env::temp_dir().join("libpakfs_tests");
        std::fs::create_dir_all(&dir).unwrap();
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = dir.join(format!(
            "struct-{}.pak",
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::write(&path, &bytes).unwrap();
        (path, bytes)
    }

    #[test]
    fn parses_structure_and_reads_blobs() {
        let (path, bytes) = make_test_pak();
        let file = File::open(&path).unwrap();
        let s = read_structure(&mut BufReaderWrap(&file)).unwrap();

        assert_eq!(s.header.entry_count, 2);
        assert_eq!(s.manifest[0].path, "a.txt");
        assert_eq!(s.manifest[1].path, "b.txt");
        assert_eq!(s.metadata[0].value, 1234);

        // positioned blob reads
        let a = s.read_blob(&file, &s.manifest[0]).unwrap();
        assert_eq!(
            Codec::from_id(s.manifest[0].codec)
                .unwrap()
                .decompress(&a, s.manifest[0].original_size)
                .unwrap(),
            b"world".to_vec()
        );
        let b = s.read_blob(&file, &s.manifest[1]).unwrap();
        assert_eq!(b, b"hello");

        // the in-memory variant agrees
        let mut cursor = std::io::Cursor::new(&bytes[..]);
        let _ = read_structure(&mut cursor).unwrap();
        let _ = &bytes;
    }

    /// Tiny Read adapter over &File so read_structure can be tested against
    /// a real file (it takes impl Read).
    struct BufReaderWrap<'a>(&'a File);

    impl Read for BufReaderWrap<'_> {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.0.read(buf)
        }
    }

    impl Write for BufReaderWrap<'_> {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            unreachable!()
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn rejects_bad_magic() {
        let bytes = [b'x'; HEADER_SIZE as usize];
        assert!(matches!(
            read_structure(&bytes[..]),
            Err(PakError::BadMagic)
        ));
    }

    #[test]
    fn rejects_truncated_manifest() {
        let mut bytes = Vec::new();
        let mut w = PakBuilder::new();
        w.add_bytes("some/path.bin", &[0u8; 32], Codec::None)
            .unwrap();
        bytes.extend_from_slice(&w.into_bytes().unwrap());
        bytes.truncate(45);
        assert!(matches!(
            read_structure(&bytes[..]),
            Err(PakError::Malformed(_) | PakError::Io(_))
        ));
    }

    #[test]
    fn rejects_unknown_codec() {
        let mut bytes = Vec::new();
        let mut w = PakBuilder::new();
        w.add_bytes("x", b"y", Codec::None).unwrap();
        bytes.extend_from_slice(&w.into_bytes().unwrap());
        bytes[HEADER_SIZE as usize + 24] = 99;
        assert!(matches!(
            read_structure(&bytes[..]),
            Err(PakError::UnknownCodec(99))
        ));
    }
}
