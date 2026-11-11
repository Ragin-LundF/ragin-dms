use std::path::Path;

use image::DynamicImage;
use pdfium_render::prelude::PdfDocument;

use crate::document::{page_of, page_text, pages_in, render_rgb};
use crate::engine::PdfEngine;
use crate::error::PdfError;
use crate::images::{self, PageImage, VISION_MAX_SIDE};

const TEXT_PAGES: usize = 10;
const VISION_PAGES: usize = 2;
const VISION_DPI: f32 = 150.0;
/// An A4 page at this resolution is about 400 pixels wide, enough for any thumbnail.
const THUMBNAIL_DPI: f32 = 48.0;
/// Fewer characters than this is a scan with at most a page number or a stamp as text.
const MIN_TEXT: usize = 80;

/// A PDF for the AI: its text layer, or, for a scan, its first pages as pictures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentContent {
    Text(String),
    PageImages(Vec<PageImage>),
    NotReadable(String),
}

impl PdfEngine {
    /// The whole text layer, pages separated by a line break, trimmed; empty for a scan.
    ///
    /// # Errors
    /// [`PdfError::NotFound`] or [`PdfError::UnsupportedFormat`] if the file cannot be opened.
    pub fn text(self, path: &Path) -> Result<String, PdfError> {
        let document = self.open(path)?;
        Ok(text_of(&document, usize::MAX)?.trim().to_string())
    }

    /// The text of every page, each trimmed.
    ///
    /// # Errors
    /// [`PdfError::NotFound`] or [`PdfError::UnsupportedFormat`] if the file cannot be opened.
    pub fn page_texts(self, path: &Path) -> Result<Vec<String>, PdfError> {
        let document = self.open(path)?;
        document.pages().iter().map(|page| Ok(normalized(&page_text(&page)?).trim().to_string())).collect()
    }

    /// The text layer of the first ten pages, cut after [`limit`] characters, or, for a scan (or with
    /// [`prefer_vision`]), the first two pages rendered at 150 dpi as JPEG of at most 1600 pixels.
    ///
    /// # Errors
    /// [`PdfError::NotFound`] or [`PdfError::UnsupportedFormat`] if the file cannot be opened or rendered.
    pub fn read_content(self, path: &Path, prefer_vision: bool, limit: usize) -> Result<DocumentContent, PdfError> {
        let document = self.open(path)?;
        let text = if prefer_vision { String::new() } else { text_of(&document, TEXT_PAGES)?.trim().to_string() };
        if text.chars().count() >= MIN_TEXT {
            return Ok(DocumentContent::Text(text.chars().take(limit).collect()));
        }
        let count = pages_in(&document);
        if count == 0 {
            return Ok(DocumentContent::NotReadable("The PDF has no pages".into()));
        }
        let pages = (0..count.min(VISION_PAGES))
            .map(|index| {
                let page = page_of(&document, index).ok_or(PdfError::NotFound)?;
                let image = render_rgb(&page, VISION_DPI)?;
                images::vision_jpeg(&DynamicImage::ImageRgb8(image), VISION_MAX_SIDE)
            })
            .collect::<Result<_, _>>()?;
        Ok(DocumentContent::PageImages(pages))
    }

    /// The first page as JPEG of at most [`max_side`] pixels, rendered at 48 dpi; `Ok(None)` for a PDF
    /// without pages. The previewer turns every error into "no thumbnail".
    ///
    /// # Errors
    /// [`PdfError::NotFound`] or [`PdfError::UnsupportedFormat`] if the file cannot be opened or rendered.
    pub fn thumbnail(self, path: &Path, max_side: u32) -> Result<Option<PageImage>, PdfError> {
        let document = self.open(path)?;
        let Some(page) = page_of(&document, 0) else {
            return Ok(None);
        };
        let image = render_rgb(&page, THUMBNAIL_DPI)?;
        Ok(Some(images::thumbnail_jpeg(&DynamicImage::ImageRgb8(image), max_side)?))
    }
}

fn text_of(document: &PdfDocument, pages: usize) -> Result<String, PdfError> {
    let mut text = String::new();
    for page in document.pages().iter().take(pages) {
        text.push_str(&normalized(&page_text(&page)?));
        text.push('\n');
    }
    Ok(text)
}

/// PDFium ends lines with `\r\n`; PDFBox, and so the rest of the app, with `\n`.
fn normalized(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}
