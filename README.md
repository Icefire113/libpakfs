# libpakfs

A Rust library for reading and writing pak files: a write-once, read-many
archive format intended as a build-system artifact, e.g. packing a game's
release assets into a single file.

Paks are frozen after creation: you build one with `PakBuilder`, then read it
with `PakFile`. Modifying contents means building a new pak.

## Features

- **Map-like reads**: give a path, get bytes: `pak.get("textures/wall.png")`.
  No offsets, handles, or format details leak into your code.
- **Lazy loading**: opening a pak only parses the header and manifest; file
  contents are read on demand via positioned reads (`pread`/`seek_read`), so
  paks can be far larger than RAM and reads are safe from multiple threads.
- **Per-file compression**: choose a codec per entry at build time:
  raw, zstd, or LZ4 (with build-time compression levels). Decompression is
  transparent to readers.
- **Pak-level metadata**: typed key/value pairs (e.g. `ModifiedAt`, `ToolId`).
- **Deterministic output**: the manifest is sorted by path, so builds are
  reproducible and diff-friendly in version control.

## Usage

```rust
use libpakfs::{
    PakFile,
    serialization::builder::PakBuilder,
    serialization::pakfile::{Codec, MetaKey},
};

// Build a pak (build-time step)
let mut b = PakBuilder::new();
b.add_bytes("hello.txt", b"Hello, world!", Codec::Zstd(3))?;
b.add_file("data.bin", File::open("data.bin")?, Codec::Lz4(0))?;
b.set_metadata(MetaKey::ModifiedAt, 1_790_000_000);
b.save("assets.pak")?;

// Read it back (runtime step)
let pak = PakFile::open("assets.pak")?;
assert_eq!(pak.get("hello.txt")?, b"Hello, world!".to_vec());
assert!(pak.exists("data.bin"));
println!("entries: {}", pak.len());
```

## Format

The on-disk format is specified in [`docs/pakfile.md`](docs/pakfile.md), with
an [ImHex pattern](docs/pakfile.hexpat) for inspecting pak files:

```
header (40 bytes) | manifest (sorted by path) | metadata | data region
```

All integers are little-endian, all strings UTF-8.
