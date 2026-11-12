//! Reading a document for the AI and for search.

mod e_invoice_content;
mod eml_mail_converter;
pub(crate) mod image_content;
mod office_content;

use std::sync::Arc;

use dms_application::port::ai::{DocumentContent, DocumentContentReader};
use dms_application::port::state::AiSettings;
use dms_domain::bill::EInvoice;
use dms_domain::document::{Document, DocumentKind};
use dms_domain::library::LibraryFolder;
use dms_domain::text;

pub use eml_mail_converter::EmlMailConverter;

use super::{LibraryLocations, convert};
use crate::PdfEngine;

/// The AI settings of this device, read for every document: "read PDFs as images" and the input budget.
pub type SettingsSource = Arc<dyn Fn() -> AiSettings + Send + Sync>;

/// Reads a document from a library folder for the classifier and for search: PDFs through the engine, JPEG and PNG
/// downscaled, the text of DOCX, PPTX and XLSX; anything else is not readable. The file is never changed.
pub struct FileContentReader {
    engine: PdfEngine,
    settings: SettingsSource,
    locations: LibraryLocations,
}

impl FileContentReader {
    #[must_use]
    pub fn new(engine: PdfEngine, settings: SettingsSource) -> Self {
        Self { engine, settings, locations: LibraryLocations }
    }
}

impl DocumentContentReader for FileContentReader {
    fn read(&self, folder: &LibraryFolder, document: &Document) -> DocumentContent {
        let settings = (self.settings)();
        let limit = usize::try_from(settings.input_characters).unwrap_or(0);
        let Some(file) = self.locations.existing(folder, &document.path) else {
            return DocumentContent::NotReadable(format!("The file is missing in {}", folder.display_name));
        };
        match document.kind() {
            DocumentKind::Pdf => match self.engine.read_content(&file, settings.prefer_vision_for_pdfs, limit) {
                Ok(content) => convert::content(content),
                Err(error) => DocumentContent::NotReadable(error.to_string()),
            },
            DocumentKind::Image => image_content::read(&file),
            DocumentKind::Office => match office_content::text(&file) {
                Ok(text) if text.is_empty() => DocumentContent::NotReadable("The file contains no text".to_owned()),
                Ok(all) => DocumentContent::Text(text::take_utf16(&all, limit).to_owned()),
                Err(reason) => DocumentContent::NotReadable(reason),
            },
            DocumentKind::Other => DocumentContent::NotReadable("This file type is not sent to the AI".to_owned()),
        }
    }

    fn text(&self, folder: &LibraryFolder, document: &Document) -> Option<String> {
        let file = self.locations.existing(folder, &document.path)?;
        let text = match document.kind() {
            DocumentKind::Pdf => self.engine.text(&file).ok()?,
            DocumentKind::Office => office_content::text(&file).ok()?,
            DocumentKind::Image | DocumentKind::Other => return None,
        };
        (!text.is_empty()).then_some(text)
    }

    fn read_e_invoice(&self, folder: &LibraryFolder, document: &Document) -> Option<EInvoice> {
        if document.kind() != DocumentKind::Pdf {
            return None;
        }
        e_invoice_content::read(self.engine, &self.locations.existing(folder, &document.path)?)
    }
}

#[cfg(test)]
pub(crate) use e_invoice_content::parse as parse_e_invoice;
