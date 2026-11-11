use dms_application::port::ai::{DocumentPreviewer, PageImage};
use dms_domain::document::DocumentKind;
use dms_domain::library::{LibraryFolder, RelativePath};

use super::content::image_content;
use super::{LibraryLocations, convert};
use crate::PdfEngine;

/// Thumbnails of PDFs and images in a library folder. Office files and other types have none, and neither has a
/// file that cannot be read.
pub struct FilePreviewer {
    engine: PdfEngine,
    locations: LibraryLocations,
}

impl FilePreviewer {
    #[must_use]
    pub fn new(engine: PdfEngine) -> Self {
        Self { engine, locations: LibraryLocations }
    }
}

impl DocumentPreviewer for FilePreviewer {
    fn thumbnail(&self, folder: &LibraryFolder, path: &RelativePath, max_side: u32) -> Option<PageImage> {
        let file = self.locations.existing(folder, path)?;
        match DocumentKind::of(path) {
            DocumentKind::Pdf => self.engine.thumbnail(&file, max_side).ok().flatten().map(convert::page_image),
            DocumentKind::Image => image_content::thumbnail(&file, max_side),
            DocumentKind::Office | DocumentKind::Other => None,
        }
    }
}
