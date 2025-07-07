use std::fs::File;

use libpakfs::serialization::deserializer::PakFileDeserializer;

fn main() {
    let mut _pk_file_deser = PakFileDeserializer::new(File::open("test.pak").unwrap());

    match _pk_file_deser.deserialize() {
        Ok(_) => (),
        Err(e) => {
            eprintln!("error: {}", e);
        }
    };
}
