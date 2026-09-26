#pragma once

// libpakfs pak file format, see pakfile.md for the full spec.
// All integers little-endian.

enum Codec : u8 {
    None = 0,
    Zstd = 1,
    Lz4  = 2,
};

struct ManifestEntry {
    u64 offset;             // relative to data_offset
    u64 compressed_size;    // stored blob size
    u64 original_size;      // size after decompression
    Codec codec;
    u16 path_len;
    char path[path_len];
};

struct Header {
    char magic[4];          // "pkfs"
    u64 entry_count;
    u64 data_offset;        // from start of file
    u16 manifest_size;      // byte size of manifest section
    u16 reserved;           // must be 0
};

Header header @ 0x00;

ManifestEntry manifest[header.entry_count] @ 0x18;

// individual file blobs, positioned relative to data_offset
fn blob(ManifestEntry e) {
    u8 data[e.compressed_size] @ header.data_offset + e.offset;
};
