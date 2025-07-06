use std::{
    fs::File,
    io::{self, BufReader, Read},
};

use crate::pakfile::pakfile::PAKFILE_MAGIC;

#[derive(Debug)]
pub struct PakFS {
    file: Option<File>,
    is_finalized: bool,
}

impl PakFS {
    pub fn new() -> Self {
        PakFS {
            file: None,
            is_finalized: false,
        }
    }

    // TODO: Stop using Strings for errors
    pub fn set_pak_file<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<(), String> {
        let file = File::open(path);

        match file {
            Ok(file) => {
                self.file = Some(file);
                Ok(())
            }
            Err(err) => Err(err.to_string()),
        }
    }

    /// Validates the pak file header, returns an `io::error` if the file cannot be read,
    /// and `Ok(true)` if the header is valid, and `Ok(false)` if the header is not valid
    fn validate_pak_file_header(&mut self) -> Result<bool, io::Error> {
        let mut x = [0_u8; 4];
        let mut reader = BufReader::new(self.file.as_mut().unwrap());
        match reader.read_exact(x.as_mut()) {
            Ok(_) => {
                dbg!(format!(
                    "File Magic: [{}, {}, {}, {}]",
                    x[0] as char, x[1] as char, x[2] as char, x[3] as char
                ));
                if PAKFILE_MAGIC == x {
                    return Ok(true);
                } else {
                    return Ok(false);
                }
            }
            Err(e) => return Err(e),
        };
    }
}
