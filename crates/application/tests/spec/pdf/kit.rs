use std::ops::Deref;
use std::sync::Arc;

use dms_application::pdf::rebuild::{KeepRebuild, RebuildAvailability, RebuildPdf};
use dms_application::pdf::{
    EditPages, EmbedPdfMetadata, MakeAllSearchable, MakeSearchable, MergeDocuments, PdfVersions, ScansWithoutText,
    SplitDocument,
};

use crate::support::TestLibrary;
use crate::support::fake::pdf::{FakeLayoutReader, FakeOcr, FakePdfEditor, FakePdfRebuilder};

/// A `TestLibrary` with the PDF fakes, wired with every pdf use case. `scans_without_text` keeps its cache, so it is
/// built once.
pub struct PdfKit {
    pub library: TestLibrary,
    pub pdf: Arc<FakePdfEditor>,
    pub ocr: Arc<FakeOcr>,
    pub rebuilder: Arc<FakePdfRebuilder>,
    pub layout_reader: Arc<FakeLayoutReader>,
    pub scans_without_text: ScansWithoutText,
}

impl Deref for PdfKit {
    type Target = TestLibrary;

    fn deref(&self) -> &TestLibrary {
        &self.library
    }
}

impl PdfKit {
    pub fn new() -> Self {
        let library = TestLibrary::new();
        let pdf = Arc::new(FakePdfEditor::new(library.folders.clone()));
        Self {
            ocr: Arc::new(FakeOcr::new(library.folders.clone())),
            rebuilder: Arc::new(FakePdfRebuilder::new(library.folders.clone())),
            layout_reader: Arc::new(FakeLayoutReader::default()),
            scans_without_text: ScansWithoutText::new(library.store.clone(), pdf.clone()),
            pdf,
            library,
        }
    }

    fn versions(&self) -> Arc<PdfVersions> {
        let library = &self.library;
        Arc::new(PdfVersions::new(
            library.store.clone(),
            library.folders.clone(),
            library.replicator.clone(),
            library.reader.clone(),
            library.folders.clone(),
            library.ids.clone(),
        ))
    }

    pub fn edit_pages(&self) -> EditPages {
        EditPages::new(self.store.clone(), self.pdf.clone(), self.versions(), self.activity.clone())
    }

    pub fn split_document(&self) -> SplitDocument {
        SplitDocument::new(
            self.store.clone(),
            self.pdf.clone(),
            self.replicator.clone(),
            self.names.clone(),
            Arc::new(self.trash_document()),
            self.activity.clone(),
            self.clock.clone(),
            self.ids.clone(),
        )
    }

    pub fn merge_documents(&self) -> MergeDocuments {
        MergeDocuments::new(
            self.store.clone(),
            self.pdf.clone(),
            self.replicator.clone(),
            self.names.clone(),
            Arc::new(self.trash_document()),
            self.activity.clone(),
            self.clock.clone(),
            self.ids.clone(),
        )
    }

    pub fn make_searchable(&self) -> MakeSearchable {
        MakeSearchable::new(
            self.store.clone(),
            self.pdf.clone(),
            self.ocr.clone(),
            self.folders.clone(),
            self.versions(),
            self.activity.clone(),
        )
    }

    pub fn make_all_searchable(&self) -> MakeAllSearchable {
        MakeAllSearchable::new(self.store.clone(), Arc::new(self.make_searchable()))
    }

    pub fn embed_pdf_metadata(&self) -> EmbedPdfMetadata {
        EmbedPdfMetadata::new(self.store.clone(), self.pdf.clone(), self.versions())
    }

    pub fn rebuild_availability(&self) -> RebuildAvailability {
        RebuildAvailability::new(self.store.clone(), self.pdf.clone())
    }

    pub fn rebuild_pdf(&self) -> RebuildPdf {
        RebuildPdf::new(
            self.store.clone(),
            self.pdf.clone(),
            self.rebuilder.clone(),
            self.layout_reader.clone(),
            Arc::new(self.rebuild_availability()),
        )
    }

    pub fn keep_rebuild(&self) -> KeepRebuild {
        KeepRebuild::new(
            self.store.clone(),
            self.rebuilder.clone(),
            self.versions(),
            self.replicator.clone(),
            self.names.clone(),
            self.activity.clone(),
            self.clock.clone(),
            self.ids.clone(),
        )
    }
}
