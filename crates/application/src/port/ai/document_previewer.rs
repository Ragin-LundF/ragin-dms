use dms_domain::library::{LibraryFolder, RelativePath};

use crate::port::ai::PageImage;

/// Small pictures of files for the screen: the first page of a PDF, or a photo. Never modifies the file.
pub trait DocumentPreviewer: Send + Sync {
    /// A picture at most `max_side` pixels on its long side, or `None` for a file without one.
    fn thumbnail(&self, folder: &LibraryFolder, path: &RelativePath, max_side: u32) -> Option<PageImage>;
}
