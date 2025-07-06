use std::fs::File;

#[derive(Debug)]
pub struct PakFS {
    file: Option<File>,
}

impl PakFS {
    pub fn new() -> Self {
        PakFS { file: None }
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
}
