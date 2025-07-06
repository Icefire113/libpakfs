use std::fs::File;

use libpakfs::serialization::deserializer::PakFileDeSerializer;

fn main() {
    let mut _pk_file_deser = PakFileDeSerializer::new(File::open("test.pak").unwrap());
    match _pk_file_deser.deserialize() {
        Ok(_) => (),
        Err(e) => {
            println!("error: {:?}", e);
        }
    };
}
