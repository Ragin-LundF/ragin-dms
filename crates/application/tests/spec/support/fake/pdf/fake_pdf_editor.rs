use std::sync::Arc;

use dms_application::port::ai::PageImage;
use dms_application::port::pdf::{PageChange, PdfEditor, PdfMetadata};
use dms_application::port::storage::{StorageError, StorageFailure, StorageResult};
use dms_domain::change::FileFacts;
use dms_domain::library::{LibraryFolder, RelativePath};

use crate::support::fake::storage::FakeFolders;

/// PDFs as text in `FakeFolders`: pages separated by `|`, a page's rotation as `@90`, its text layer as `#text`, and
/// embedded metadata as a trailing `;title`. Enough to see what an operation did.
pub struct FakePdfEditor {
    folders: Arc<FakeFolders>,
}

impl FakePdfEditor {
    pub fn new(folders: Arc<FakeFolders>) -> Self {
        Self { folders }
    }

    fn read(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<String> {
        self.folders.content(folder, path.value()).ok_or_else(missing)
    }

    fn write(&self, folder: &LibraryFolder, target: &RelativePath, content: &str) -> StorageResult<FileFacts> {
        if self.folders.content(folder, target.value()).is_some() {
            return Err(StorageError::new(StorageFailure::AlreadyExists));
        }
        self.folders.put(folder, target.value(), content);
        Ok(FakeFolders::facts_of(content))
    }
}

fn missing() -> StorageError {
    StorageError::new(StorageFailure::NotFound)
}

fn pages_of(content: &str) -> Vec<&str> {
    content.split(';').next().unwrap_or_default().split('|').collect()
}

impl PdfEditor for FakePdfEditor {
    fn page_count(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<usize> {
        Ok(pages_of(&self.read(folder, path)?).len())
    }

    fn render_page(
        &self,
        _folder: &LibraryFolder,
        _path: &RelativePath,
        _page: usize,
        _max_side: u32,
    ) -> Option<PageImage> {
        None
    }

    fn rewrite(
        &self,
        folder: &LibraryFolder,
        source: &RelativePath,
        pages: &[PageChange],
        target: &RelativePath,
    ) -> StorageResult<FileFacts> {
        let content = self.read(folder, source)?;
        let existing = pages_of(&content);
        let rewritten = pages
            .iter()
            .map(|change| {
                let page = existing[change.page];
                if change.rotation == 0 { page.to_owned() } else { format!("{page}@{}", change.rotation) }
            })
            .collect::<Vec<_>>()
            .join("|");
        self.write(folder, target, &rewritten)
    }

    fn merge(
        &self,
        folder: &LibraryFolder,
        sources: &[RelativePath],
        target: &RelativePath,
    ) -> StorageResult<FileFacts> {
        let contents = sources.iter().map(|source| self.read(folder, source)).collect::<StorageResult<Vec<_>>>()?;
        self.write(folder, target, &contents.join("|"))
    }

    fn has_text_layer(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<bool> {
        Ok(pages_of(&self.read(folder, path)?).iter().all(|page| page.contains('#')))
    }

    fn add_text_layer(
        &self,
        folder: &LibraryFolder,
        source: &RelativePath,
        pages: &[String],
        target: &RelativePath,
    ) -> StorageResult<FileFacts> {
        let content = self.read(folder, source)?;
        let layered = pages_of(&content)
            .iter()
            .enumerate()
            .map(|(index, page)| format!("{page}#{}", pages.get(index).map_or("", String::as_str)))
            .collect::<Vec<_>>()
            .join("|");
        self.write(folder, target, &layered)
    }

    fn write_metadata(
        &self,
        folder: &LibraryFolder,
        source: &RelativePath,
        metadata: &PdfMetadata,
        target: &RelativePath,
    ) -> StorageResult<FileFacts> {
        let content = self.read(folder, source)?;
        let body = content.split(';').next().unwrap_or_default();
        // Kotlin's string template prints a missing title as "null".
        let title = metadata.title.as_deref().unwrap_or("null");
        self.write(folder, target, &format!("{body};{title}"))
    }

    fn read_metadata(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<PdfMetadata> {
        let content = self.read(folder, path)?;
        let title = content.split_once(';').map(|(_, title)| title.to_owned()).filter(|title| !title.is_empty());
        Ok(PdfMetadata { title, ..PdfMetadata::default() })
    }
}
