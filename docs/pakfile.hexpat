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

u64 total_file_data_size = 0;
for (u64 i = 0, i<header.entry_count, i = i + 1) {
    total_file_data_size += header.manifest[i].file_size;
}

u8 data[total_file_data_size] @ $;