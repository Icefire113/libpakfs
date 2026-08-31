use std::fs::File;

use pakfs::{
    PakFile,
    serialization::{builder::PakBuilder, pakfile::Codec},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // build a small pak
    let mut b = PakBuilder::new();
    b.add_file("README.md", File::open("./README.md")?, Codec::Zstd(9))?;
    b.add_bytes("hello.txt", b"Hello from inside the pak!", Codec::None)?;
    b.add_bytes(
        "data.bin",
        (0u8..=255).collect::<Vec<u8>>().as_slice(),
        Codec::Lz4(0),
    )?;
    b.save("test_out.pak")?;

    // read it back
    let pak = PakFile::open("test_out.pak")?;
    println!("entries: {}", pak.len());
    println!("hello.txt: {}", String::from_utf8(pak.get("hello.txt")?)?);
    println!("data.bin len: {}", pak.get("data.bin")?.len());
    println!("exists data.bin: {}", pak.exists("data.bin"));

    Ok(())
}
