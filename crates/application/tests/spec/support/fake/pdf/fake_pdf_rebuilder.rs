use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use dms_application::port::ai::PageImage;
use dms_application::port::pdf::{PageInput, PdfRebuilder, PreparedPage, RebuiltFile};
use dms_application::port::storage::{StorageError, StorageFailure, StorageResult};
use dms_domain::change::FileFacts;
use dms_domain::document::ContentHash;
use dms_domain::library::{LibraryFolder, RelativePath};
use dms_domain::rebuild::{KeepReason, LayoutBox, RebuiltPage};
use indexmap::IndexMap;

use crate::support::Shared;
use crate::support::fake::storage::FakeFolders;

/// Rebuilds as text, waiting outside the library in `waiting`: a read page becomes its line texts joined by `/`, with
/// `[image]` for a kept line, and a failed page becomes `page n kept`. A prepared page's image is `page n`, a crop's
/// image is its box (`{:?}`), so a fake reader can tell them apart.
pub struct FakePdfRebuilder {
    folders: Arc<FakeFolders>,
    pub waiting: Shared<HashMap<String, String>>,
    next: AtomicUsize,
}

impl FakePdfRebuilder {
    pub fn new(folders: Arc<FakeFolders>) -> Self {
        Self { folders, waiting: Shared::default(), next: AtomicUsize::new(0) }
    }

    /// The image text of a crop of `bounds`.
    pub fn crop_text(bounds: &LayoutBox) -> String {
        format!("{bounds:?}")
    }
}

fn image_of(text: &str) -> PageImage {
    PageImage { bytes: text.as_bytes().to_vec(), mime_type: "image/png".to_owned() }
}

fn summary_of(page: &PageInput) -> RebuiltPage {
    let lines = match page {
        PageInput::Read { layout, .. } => layout.lines.as_slice(),
        PageInput::Failed { .. } => &[],
    };
    let failed = matches!(page, PageInput::Failed { .. }).then_some(KeepReason::PageFailed);
    let mut kept_reasons: IndexMap<KeepReason, usize> = IndexMap::new();
    for reason in lines.iter().filter_map(|line| line.kept).chain(failed) {
        *kept_reasons.entry(reason).or_default() += 1;
    }
    RebuiltPage {
        lines_as_text: lines.iter().filter(|line| line.kept.is_none()).count(),
        lines_total: lines.len(),
        angle: 0.0,
        kept_reasons,
        regions: Vec::new(),
    }
}

impl PdfRebuilder for FakePdfRebuilder {
    fn prepare_page(&self, folder: &LibraryFolder, path: &RelativePath, page: usize) -> StorageResult<PreparedPage> {
        self.folders.content(folder, path.value()).ok_or_else(|| StorageError::new(StorageFailure::NotFound))?;
        Ok(PreparedPage { page_index: page, image: image_of(&format!("page {page}")), angle: 0.0 })
    }

    fn crop(&self, _prepared: &PreparedPage, bounds: &LayoutBox) -> PageImage {
        image_of(&Self::crop_text(bounds))
    }

    fn write(
        &self,
        _folder: &LibraryFolder,
        _source: &RelativePath,
        source_hash: &ContentHash,
        pages: &[PageInput],
    ) -> StorageResult<RebuiltFile> {
        let content = pages
            .iter()
            .map(|page| match page {
                PageInput::Failed { page_index } => format!("page {page_index} kept"),
                PageInput::Read { layout, .. } => layout
                    .lines
                    .iter()
                    .map(|line| if line.kept.is_none() { line.text.as_str() } else { "[image]" })
                    .collect::<Vec<_>>()
                    .join("/"),
            })
            .collect::<Vec<_>>()
            .join("|");
        let id = format!("rebuild-{}", self.next.fetch_add(1, Ordering::SeqCst));
        // Kotlin counted UTF-16 units; the tests only use ASCII here.
        let size = content.len() as u64;
        self.waiting.update(|waiting| waiting.insert(id.clone(), content));
        Ok(RebuiltFile { id, pages: pages.iter().map(summary_of).collect(), size, source_hash: source_hash.clone() })
    }

    fn render_page(&self, _rebuilt: &RebuiltFile, _page: usize, _max_side: u32) -> Option<PageImage> {
        None
    }

    fn place(&self, rebuilt: &RebuiltFile, folder: &LibraryFolder, target: &RelativePath) -> StorageResult<FileFacts> {
        let content =
            self.waiting.lock().get(&rebuilt.id).cloned().ok_or_else(|| StorageError::new(StorageFailure::NotFound))?;
        if self.folders.content(folder, target.value()).is_some() {
            return Err(StorageError::new(StorageFailure::AlreadyExists));
        }
        self.folders.put(folder, target.value(), &content);
        Ok(FakeFolders::facts_of(&content))
    }

    fn discard(&self, rebuilt: &RebuiltFile) {
        self.waiting.update(|waiting| waiting.remove(&rebuilt.id));
    }
}
