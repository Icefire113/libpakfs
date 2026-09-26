# PakFile Format & API Specification (v1)

Pak files are write-once, read-many archives intended as build-system artifacts
(e.g. game-engine release assets). They are **never modified after creation**;
changing contents means building a new pak file.

All integers are **little-endian**. All strings are **UTF-8**.

## 1. File Layout

A pak file consists of three regions, in order:

```
+------------------+
| Header           |
| Manifest         |
| Data region      |
+------------------+
```

### 1.1 Header

Fixed-size, always at offset 0. There is **no version field**: when the format
changes, the format itself is replaced (magic stays `pkfs`; incompatible
readers error on malformed structure, not on version).

| Offset | Size | Type      | Field          | Description                                    |
|-------:|-----:|-----------|----------------|------------------------------------------------|
| 0      | 4    | `[u8; 4]` | magic          | Must be `"pkfs"` (ASCII, not null-terminated)  |
| 4      | 8    | `u64`     | entry_count    | Number of manifest entries                     |
| 12     | 8    | `u64`     | data_offset    | Byte offset from start of file to data region  |
| 20     | 2    | `u16`     | manifest_size  | Total byte size of the manifest section        |
| 22     | 2    | reserved  | reserved       | Must be 0; pad header to 24 bytes              |

Total header size: **24 bytes**.

### 1.2 Manifest

A sequence of `entry_count` manifest entries, sorted **lexicographically by
path** (byte-wise UTF-8 comparison). Sorting is required so that builds are
deterministic, and so readers may binary-search instead of materializing a map
(a map/cache is still allowed).

Each entry:

| Size  | Type   | Field           | Description                                              |
|------:|--------|-----------------|----------------------------------------------------------|
| 8     | `u64`  | offset          | Byte offset of the file's data **relative to `data_offset`** |
| 8     | `u64`  | compressed_size | Size of the stored (possibly compressed) blob in bytes   |
| 8     | `u64`  | original_size   | Size of the file after decompression                     |
| 1     | `u8`   | codec           | Compression codec id (see §1.4)                          |
| 2     | `u16`  | path_len        | Length of `path` in bytes                                |
| var   | bytes  | path            | UTF-8 path, no trailing null; delimited by `path_len`    |

Duplicate paths are invalid. Paths use `/` as the separator. Paths are
case-sensitive. The empty path is invalid.

### 1.3 Data region

The concatenation of every entry's stored blob. Blob `i` occupies bytes
`[data_offset + entry_i.offset, data_offset + entry_i.offset + entry_i.compressed_size)`.
Blobs are tightly packed (no alignment/padding) in manifest order.

For an uncompressed entry (`codec = 0`), `compressed_size == original_size` and
the blob is the file verbatim.

### 1.4 Codec ids

| Id | Codec                          |
|---:|--------------------------------|
| 0  | None (stored raw)              |
| 1  | Zstd (standard frame format)   |
| 2  | LZ4 (LZ4 frame format)         |

Compression levels for zstd and LZ4 are build-time choices and are **not**
stored in the file; only the codec id matters to a reader.

Readers must error on unknown codec ids.

## 2. API Specification

The public API is split into a read-only runtime type and a consuming builder.

### 2.1 `PakFile` — runtime reader

Read-only, map-like semantics. Users never see offsets, handles, or format
details; they give a path, they get bytes.

```rust
impl PakFile {
    /// Opens a pak file, reads the header and manifest, and caches the
    /// path -> entry table in memory. The underlying file stays open for
    /// positioned reads.
    fn open(path: impl AsRef<Path>) -> Result<PakFile, PakError>;

    /// True if `path` exists in the pak.
    fn exists(&self, path: &str) -> bool;

    /// Number of entries.
    fn len(&self) -> usize;

    /// Returns the full, decompressed contents of `path`.
    /// Errors with `NotFound` if the path is not in the pak.
    fn get(&self, path: &str) -> Result<Vec<u8>, PakError>;

    /// Reads the full, decompressed contents of `path` into `buf`.
    /// Errors with `BufferTooSmall` if `buf.len() < size(path)`;
    /// never truncates silently.
    fn read_into(&self, path: &str, buf: &mut [u8]) -> Result<(), PakError>;

    /// Uncompressed size of `path` in bytes.
    fn size(&self, path: &str) -> Result<u64, PakError>;

    /// All paths in the pak, sorted.
    fn paths(&self) -> impl Iterator<Item = &str>;
}
```

Implementation notes (not part of the public contract):

- The manifest is parsed once at `open()` and cached as a map
  (`HashMap`/`BTreeMap`) from path to entry (offset, sizes, codec).
- File data is read via **positioned reads** (pread/seek_read), never a shared
  cursor, so reads are safe from multiple threads concurrently.
- Decompression is transparent to callers.

### 2.2 `PakBuilder` — build side

Stages files in memory-of-intent (sources may be streamed), then `save`
consumes the builder and writes the pak. A saved pak is frozen.

```rust
impl PakBuilder {
    fn new() -> Self;

    /// Stages a file, streaming from `src`. `codec` selects compression
    /// applied to this entry only.
    fn add_file(&mut self, path: &str, src: impl Read, codec: Codec)
        -> Result<&mut Self, PakError>;

    /// Stages bytes directly. `codec` selects compression for this entry.
    fn add_bytes(&mut self, path: &str, bytes: &[u8], codec: Codec)
        -> Result<&mut Self, PakError>;

    /// Consumes the builder and writes the pak file. Writes the manifest
    /// sorted by path; duplicate paths are an error.
    fn save(self, out: impl AsRef<Path>) -> Result<(), PakError>;
}
```

(In the current implementation, `add_file` reads the source fully into
memory before staging; a streaming variant is planned.)

### 2.3 Shared types

```rust
/// Per-entry compression codec, chosen at build time.
pub enum Codec {
    None,
    Zstd(u8),   // level; build-time choice, not stored in file
    Lz4(u8),    // level; build-time choice, not stored in file
}
```

### 2.4 Errors

A single `PakError` enum covering at least:

- `Io(std::io::Error)` — underlying file I/O
- `NotFound` — path not present in the pak
- `BufferTooSmall { needed: u64, got: usize }` — `read_into` with a too-small buffer
- `BadMagic` — file does not start with `pkfs`
- `Malformed(&'static str)` — structurally invalid header/manifest
- `UnknownCodec(u8)` — manifest references an unknown codec id
- `DuplicatePath(String)` — build-time duplicate
- `Compression(...)` — codec library errors

## 3. Non-goals (v1)

- No version field; no backward compatibility story for format changes.
- No per-file or pak-level metadata, and no checksums.
- No modification/appending of existing pak files.
- No memory-map alignment guarantees (data region is unaligned, tightly packed).
- No partial/offset reads: users slice the returned `Vec` themselves.
