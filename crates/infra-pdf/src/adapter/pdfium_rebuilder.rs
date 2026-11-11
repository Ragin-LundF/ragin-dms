use std::path::PathBuf;

use dms_application::port::ai::PageImage;
use dms_application::port::pdf::{PageInput, PdfRebuilder, PreparedPage, RebuiltFile};
use dms_application::port::storage::StorageResult;
use dms_domain::change::FileFacts;
use dms_domain::document::ContentHash;
use dms_domain::library::{LibraryFolder, RelativePath};
use dms_domain::rebuild::LayoutBox;

use super::LibraryLocations;
use super::convert::{self, not_found, storage_error, unreachable};
use crate::{PdfEngine, Rebuilder};

/// "Convert to real PDF" on PDFium. Rebuilt PDFs wait in the work folder, in the app's own data, until they are
/// placed or discarded; make the adapter once when the app starts, which is when leftovers are deleted.
pub struct PdfiumRebuilder {
    rebuilder: Rebuilder,
    locations: LibraryLocations,
}

impl PdfiumRebuilder {
    #[must_use]
    pub fn new(engine: PdfEngine, work_directory: PathBuf) -> Self {
        Self { rebuilder: Rebuilder::new(engine, work_directory), locations: LibraryLocations }
    }
}

impl PdfRebuilder for PdfiumRebuilder {
    fn prepare_page(&self, folder: &LibraryFolder, path: &RelativePath, page: usize) -> StorageResult<PreparedPage> {
        let file = self.locations.existing(folder, path).ok_or_else(not_found)?;
        self.rebuilder.prepare_page(&file, page).map(convert::prepared_from_engine).map_err(storage_error)
    }

    /// An empty picture if `prepared` is no readable picture, which [`Self::prepare_page`] never makes; reading it
    /// fails, and the line stays an image.
    fn crop(&self, prepared: &PreparedPage, bounds: &LayoutBox) -> PageImage {
        self.rebuilder
            .crop(&convert::prepared_to_engine(prepared), convert::layout_box(bounds))
            .map_or_else(|_| PageImage { bytes: Vec::new(), mime_type: "image/png".to_owned() }, convert::page_image)
    }

    fn write(
        &self,
        folder: &LibraryFolder,
        source: &RelativePath,
        source_hash: &ContentHash,
        pages: &[PageInput],
    ) -> StorageResult<RebuiltFile> {
        let file = self.locations.existing(folder, source).ok_or_else(not_found)?;
        let pages: Vec<_> = pages.iter().map(convert::page_input).collect();
        let rebuilt = self.rebuilder.write(&file, source_hash.value(), &pages).map_err(storage_error)?;
        Ok(RebuiltFile {
            id: rebuilt.id,
            pages: rebuilt.pages.into_iter().map(convert::rebuilt_page).collect(),
            size: rebuilt.size,
            source_hash: source_hash.clone(),
        })
    }

    fn render_page(&self, rebuilt: &RebuiltFile, page: usize, max_side: u32) -> Option<PageImage> {
        let handle = convert::rebuilt_handle(rebuilt);
        self.rebuilder.render_page(&handle, page, max_side).ok().flatten().map(convert::page_image)
    }

    fn place(&self, rebuilt: &RebuiltFile, folder: &LibraryFolder, target: &RelativePath) -> StorageResult<FileFacts> {
        let output = self.locations.target(folder, target).ok_or_else(unreachable)?;
        self.rebuilder.place(&convert::rebuilt_handle(rebuilt), &output).map(convert::file_facts).map_err(storage_error)
    }

    fn discard(&self, rebuilt: &RebuiltFile) {
        self.rebuilder.discard(&convert::rebuilt_handle(rebuilt));
    }
}
