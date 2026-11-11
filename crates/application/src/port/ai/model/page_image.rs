use std::fmt;

/// One rendered page or image, already downscaled for the model.
#[derive(Clone)]
pub struct PageImage {
    pub bytes: Vec<u8>,
    pub mime_type: String,
}

/// Prints the size instead of the bytes, so logging a request stays readable.
impl fmt::Debug for PageImage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PageImage")
            .field("bytes", &format_args!("<{} bytes>", self.bytes.len()))
            .field("mime_type", &self.mime_type)
            .finish()
    }
}
