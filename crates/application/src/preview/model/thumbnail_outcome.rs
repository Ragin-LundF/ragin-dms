use crate::port::ai::PageImage;

#[derive(Debug, Clone)]
pub enum ThumbnailOutcome {
    Rendered(PageImage),
    /// The file has no picture, or it cannot be read.
    NoPicture,
    /// No library is open, or the folder is not one of its folders.
    FolderNotFound,
}
