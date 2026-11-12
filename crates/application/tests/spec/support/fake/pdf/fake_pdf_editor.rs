use std::sync::Arc;

use dms_application::port::ai::PageImage;
use dms_application::port::pdf::{PageChange, PdfEditor, PdfMetadata, TextBlock, TextDocument};
use dms_application::port::storage::{StorageError, StorageFailure, StorageResult};
use dms_domain::change::FileFacts;
use dms_domain::library::{LibraryFolder, RelativePath};

use crate::support::fake::storage::FakeFolders;

/// PDFs as text in `FakeFolders`: pages separated by `|`, a page's rotation as `@90`, its text layer as `#text`, and
/// embedded metadata as a trailing `;title`. A page whose picture part is `blank` looks blank; a rendered page is its
/// picture part as bytes. A text document is its blocks as lines: `# heading`, paragraphs, `key: value`,
/// table cells separated by tabs, and an empty line per spacer. Enough to see what an operation did.
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
    /// A file starting with `BROKEN` is no PDF.
    fn page_count(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<usize> {
        let content = self.read(folder, path)?;
        if content.starts_with("BROKEN") {
            return Err(StorageError::new(StorageFailure::UnsupportedFormat));
        }
        Ok(pages_of(&content).len())
    }

    fn render_page(
        &self,
        folder: &LibraryFolder,
        path: &RelativePath,
        page: usize,
        _max_side: u32,
    ) -> Option<PageImage> {
        let content = self.read(folder, path).ok()?;
        let picture = picture_of(pages_of(&content).get(page)?);
        Some(PageImage { bytes: picture.as_bytes().to_vec(), mime_type: "image/png".to_owned() })
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

    fn page_texts(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<Vec<String>> {
        let content = self.read(folder, path)?;
        Ok(pages_of(&content).iter().map(|page| page.split_once('#').map_or("", |(_, text)| text).to_owned()).collect())
    }

    fn blank_pages(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<Vec<bool>> {
        Ok(pages_of(&self.read(folder, path)?).iter().map(|page| picture_of(page) == "blank").collect())
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

    fn write_text_document(
        &self,
        folder: &LibraryFolder,
        target: &RelativePath,
        document: &TextDocument,
    ) -> StorageResult<FileFacts> {
        let lines: Vec<String> = document.blocks.iter().map(text_of).collect();
        self.write(folder, target, &lines.join("\n"))
    }

    fn text_document_bytes(&self, document: &TextDocument) -> StorageResult<Vec<u8>> {
        let lines: Vec<String> = document.blocks.iter().map(text_of).collect();
        Ok(lines.join("\n").into_bytes())
    }
}

/// A page without its text layer and rotation.
fn picture_of(page: &str) -> &str {
    page.split(['#', '@']).next().unwrap_or_default()
}

fn text_of(block: &TextBlock) -> String {
    match block {
        TextBlock::Heading(text) => format!("# {text}"),
        TextBlock::Paragraph(text) => text.clone(),
        TextBlock::KeyValueTable(rows) => {
            rows.iter().map(|(key, value)| format!("{key}: {value}")).collect::<Vec<_>>().join("\n")
        }
        TextBlock::Table { header, rows } => {
            std::iter::once(header).chain(rows).map(|row| row.join("\t")).collect::<Vec<_>>().join("\n")
        }
        TextBlock::Spacer => String::new(),
    }
}
