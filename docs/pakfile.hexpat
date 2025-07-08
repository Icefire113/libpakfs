struct ManifestEntry {
    u64 fileOffset;
    u64 file_size;
    char file_path[];
};

struct HEADER {
    char magic[4];
    u32 version;
    u64 pakFileID;
    u64 entry_count;
    ManifestEntry manifest[entry_count];
};

HEADER header @ 0x00;