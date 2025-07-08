#[allow(dead_code)]
/// This is a file that is stored inside the pak file
/// these should not be edited, anytime an edit is made a new `InternalFile` should be created.
pub struct InternalFile {
    data: Vec<u8>,
}

impl InternalFile {
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
