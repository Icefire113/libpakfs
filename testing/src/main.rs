use pakfs::{
    PakFile,
    serialization::{builder::PakBuilder, pakfile::Codec},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // build a small pak
    let mut w = PakBuilder::new();
    w.add_bytes("hello.txt", b"Hello from inside the pak!", Codec::Zstd(3))?;
    w.add_bytes(
        "data.bin",
        (0u8..=255).collect::<Vec<u8>>().as_slice(),
        Codec::Lz4(0),
    )?;
    w.save("test_out.pak")?;

    // read it back
    let pak = PakFile::open("test_out.pak")?;
    println!("entries: {}", pak.len());
    println!("hello.txt: {}", String::from_utf8(pak.get("hello.txt")?)?);
    println!("data.bin len: {}", pak.get("data.bin")?.len());
    println!("exists data.bin: {}", pak.exists("data.bin"));

    Ok(())
}
