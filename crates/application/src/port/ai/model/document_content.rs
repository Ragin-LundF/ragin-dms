use crate::port::ai::PageImage;

/// What a document reader made of a file: text, page images, or nothing usable.
#[derive(Debug, Clone)]
pub enum DocumentContent {
    /// Text from a PDF text layer or an Office file.
    Text(String),
    /// Rendered pages of a scan, or an image file; needs a vision-capable model.
    PageImages(Vec<PageImage>),
    NotReadable(String),
}
