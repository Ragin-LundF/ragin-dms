use std::path::PathBuf;

use dms_application::port::ai::PageImage;
use dms_application::port::pdf::{PageChange, PdfEditor, PdfMetadata};
use dms_application::port::storage::StorageResult;
use dms_domain::change::FileFacts;
use dms_domain::library::{LibraryFolder, RelativePath};

use super::LibraryLocations;
use super::convert::{self, not_found, storage_error, unreachable};
use crate::PdfEngine;

/// The PDF editor on PDFium. Sources are only read; every result is a new file at a target that must not exist yet.
pub struct PdfiumEditor {
    engine: PdfEngine,
    locations: LibraryLocations,
}

impl PdfiumEditor {
    #[must_use]
    pub fn new(engine: PdfEngine) -> Self {
        Self { engine, locations: LibraryLocations }
    }

    fn source(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<PathBuf> {
        self.locations.existing(folder, path).ok_or_else(not_found)
    }

    /// The source, which must exist, and the target's place, which needs a reachable folder.
    fn source_and_target(
        &self,
        folder: &LibraryFolder,
        source: &RelativePath,
        target: &RelativePath,
    ) -> StorageResult<(PathBuf, PathBuf)> {
        let input = self.source(folder, source)?;
        let output = self.locations.target(folder, target).ok_or_else(unreachable)?;
        Ok((input, output))
    }
}

impl PdfEditor for PdfiumEditor {
    fn page_count(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<usize> {
        self.engine.page_count(&self.source(folder, path)?).map_err(storage_error)
    }

    fn render_page(
        &self,
        folder: &LibraryFolder,
        path: &RelativePath,
        page: usize,
        max_side: u32,
    ) -> Option<PageImage> {
        let file = self.locations.existing(folder, path)?;
        self.engine.render_page(&file, page, max_side).ok().flatten().map(convert::page_image)
    }

    fn rewrite(
        &self,
        folder: &LibraryFolder,
        source: &RelativePath,
        pages: &[PageChange],
        target: &RelativePath,
    ) -> StorageResult<FileFacts> {
        let (input, output) = self.source_and_target(folder, source, target)?;
        let pages: Vec<_> = pages.iter().map(convert::page_change).collect();
        self.engine.rewrite(&input, &pages, &output).map(convert::file_facts).map_err(storage_error)
    }

    fn merge(
        &self,
        folder: &LibraryFolder,
        sources: &[RelativePath],
        target: &RelativePath,
    ) -> StorageResult<FileFacts> {
        let inputs = sources.iter().map(|source| self.source(folder, source)).collect::<StorageResult<Vec<_>>>()?;
        let output = self.locations.target(folder, target).ok_or_else(unreachable)?;
        let inputs: Vec<_> = inputs.iter().map(PathBuf::as_path).collect();
        self.engine.merge(&inputs, &output).map(convert::file_facts).map_err(storage_error)
    }

    fn has_text_layer(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<bool> {
        self.engine.has_text_layer(&self.source(folder, path)?).map_err(storage_error)
    }

    fn add_text_layer(
        &self,
        folder: &LibraryFolder,
        source: &RelativePath,
        pages: &[String],
        target: &RelativePath,
    ) -> StorageResult<FileFacts> {
        let (input, output) = self.source_and_target(folder, source, target)?;
        self.engine.add_text_layer(&input, pages, &output).map(convert::file_facts).map_err(storage_error)
    }

    fn write_metadata(
        &self,
        folder: &LibraryFolder,
        source: &RelativePath,
        metadata: &PdfMetadata,
        target: &RelativePath,
    ) -> StorageResult<FileFacts> {
        let (input, output) = self.source_and_target(folder, source, target)?;
        let metadata = convert::metadata_to_engine(metadata);
        self.engine.write_metadata(&input, &metadata, &output).map(convert::file_facts).map_err(storage_error)
    }

    fn read_metadata(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<PdfMetadata> {
        self.engine.read_metadata(&self.source(folder, path)?).map(convert::metadata_from_engine).map_err(storage_error)
    }
}
