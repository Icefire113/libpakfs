use std::{fs::File, path::Path};

use libpakfs::{
    serialization::pakfile::PakFileData,
    serialization::{deserializer::PakFileDeserializer, serializer::PakFileSerializer},
};

fn main() {
    let mut pk_file_deser = PakFileDeserializer::new(File::open("test.pak").unwrap());

    match pk_file_deser.deserialize() {
        Ok(_) => (),
        Err(e) => {
            panic!("deser error: {}", e);
        }
    };

    let pk_file = PakFileData::new();

    let pk_file_ser = PakFileSerializer::new(pk_file);

    match pk_file_ser.save_to(Path::new("test2.pak")) {
        Ok(_) => (),
        Err(e) => {
            panic!("ser error: {}", e);
        }
    };
}
