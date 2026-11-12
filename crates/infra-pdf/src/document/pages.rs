use std::path::Path;

use image::{DynamicImage, Pixel, RgbImage};
use pdfium_render::prelude::{PdfDocument, PdfPage, PdfPageRenderRotation, PdfRenderConfig};

use crate::engine::PdfEngine;
use crate::error::PdfError;
use crate::files::{self, FileFacts};
use crate::images::{self, PageImage};

const POINTS_PER_INCH: f32 = 72.0;
/// Pictures for the page editor are never rendered sharper than this.
const MAX_EDITOR_DPI: f32 = 150.0;
/// A page this small is enough to tell an empty sheet from a written one.
const BLANK_CHECK_DPI: f32 = 50.0;
/// Pixels darker than this count as ink; thin text rendered small is grey, not black.
const INK_LUMA: u8 = 200;
/// Below this share of ink a page counts as blank; dust and scanner noise stay far below it, a line of text above.
const BLANK_INK_RATIO: f64 = 0.001;

/// One page of a rewritten PDF: page [`page`] of the source (counted from 0), turned clockwise by
/// [`rotation`] degrees, a multiple of 90.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageChange {
    pub page: usize,
    pub rotation: i32,
}

impl PdfEngine {
    /// # Errors
    /// [`PdfError::NotFound`] or [`PdfError::UnsupportedFormat`] if the file cannot be opened.
    pub fn page_count(self, path: &Path) -> Result<usize, PdfError> {
        Ok(pages_in(&self.open(path)?))
    }

    /// One page as a PNG picture of at most [`max_side`] pixels on its long side, for the page editor;
    /// `Ok(None)` if the page does not exist. The adapter turns every error into "no picture".
    ///
    /// # Errors
    /// [`PdfError::NotFound`] or [`PdfError::UnsupportedFormat`] if the file cannot be opened or rendered.
    pub fn render_page(self, path: &Path, page: usize, max_side: u32) -> Result<Option<PageImage>, PdfError> {
        let document = self.open(path)?;
        let Some(page) = page_of(&document, page) else {
            return Ok(None);
        };
        let longest = page.width().value.max(page.height().value);
        #[allow(clippy::cast_precision_loss)] // picture sides are far below 2^24
        let needed = max_side.max(1) as f32 * POINTS_PER_INCH / longest;
        let image = render_rgb(&page, needed.min(MAX_EDITOR_DPI))?;
        Ok(Some(images::png_of(&DynamicImage::ImageRgb8(images::scaled_down(&image, max_side.max(1))))?))
    }

    /// For every page, whether it looks blank: less than 0.1 % ink on a picture at 50 dpi, like a separator sheet or
    /// an empty back side.
    ///
    /// # Errors
    /// [`PdfError::NotFound`] or [`PdfError::UnsupportedFormat`] if the file cannot be opened or rendered.
    pub fn blank_pages(self, path: &Path) -> Result<Vec<bool>, PdfError> {
        let document = self.open(path)?;
        document
            .pages()
            .iter()
            .map(|page| Ok(ink_ratio(&render_rgb(&page, BLANK_CHECK_DPI)?) < BLANK_INK_RATIO))
            .collect()
    }

    /// A new PDF of the [`pages`] of [`source`], in their order and turned as given; this rotates,
    /// deletes, reorders and extracts. [`target`] must not exist yet.
    ///
    /// # Errors
    /// [`PdfError::InvalidRequest`] for no pages, a page beyond the end or a rotation that is no quarter
    /// turn; [`PdfError::AlreadyExists`] if [`target`] exists; the errors of opening [`source`].
    pub fn rewrite(self, source: &Path, pages: &[PageChange], target: &Path) -> Result<FileFacts, PdfError> {
        if pages.is_empty() {
            return Err(PdfError::InvalidRequest("a PDF needs at least one page".into()));
        }
        if pages.iter().any(|change| change.rotation % 90 != 0) {
            return Err(PdfError::InvalidRequest("pages turn in quarter turns".into()));
        }
        let input = self.open(source)?;
        let count = pages_in(&input);
        if pages.iter().any(|change| change.page >= count) {
            return Err(PdfError::InvalidRequest(format!("the PDF has only {count} pages")));
        }
        let mut output = self.create()?;
        // One import keeps resources shared between the pages shared; PDFium counts pages from 1 here.
        let order = pages.iter().map(|change| (change.page + 1).to_string()).collect::<Vec<_>>().join(",");
        output.pages_mut().copy_pages_from_document(&input, &order, 0)?;
        for (index, change) in pages.iter().enumerate() {
            let mut page = page_of(&output, index).ok_or(PdfError::NotFound)?;
            let current = degrees_of(page.rotation()?);
            page.set_rotation(rotation_of(current + change.rotation));
        }
        save_new(&output, target)
    }

    /// A new PDF of every page of [`sources`], one after the other. [`target`] must not exist yet.
    ///
    /// # Errors
    /// [`PdfError::InvalidRequest`] for no sources; [`PdfError::AlreadyExists`] if [`target`] exists; the
    /// errors of opening a source.
    pub fn merge(self, sources: &[&Path], target: &Path) -> Result<FileFacts, PdfError> {
        if sources.is_empty() {
            return Err(PdfError::InvalidRequest("nothing to merge".into()));
        }
        let inputs = sources.iter().map(|source| self.open(source)).collect::<Result<Vec<_>, _>>()?;
        let mut output = self.create()?;
        for input in &inputs {
            output.pages_mut().append(input)?;
        }
        save_new(&output, target)
    }
}

pub(crate) fn page_of<'a>(document: &PdfDocument<'a>, index: usize) -> Option<PdfPage<'a>> {
    let index = i32::try_from(index).ok()?;
    document.pages().get(index).ok()
}

/// Every character PDFium finds on [`page`], with the line breaks and spaces it generates. Not
/// `PdfPageText::all`, which drops text outside the unrotated page box and so everything on a turned page.
pub(crate) fn page_text(page: &PdfPage) -> Result<String, PdfError> {
    let text = page.text()?;
    Ok(text
        .chars()
        .iter()
        .filter_map(|character| character.unicode_char())
        .filter(|character| !character.is_control() || matches!(character, '\n' | '\r' | '\t'))
        .collect())
}

pub(crate) fn pages_in(document: &PdfDocument) -> usize {
    usize::try_from(document.pages().len()).unwrap_or(0)
}

/// [`page`] rendered at [`dpi`] on white, with its rotation applied, as PDFBox renders it.
pub(crate) fn render_rgb(page: &PdfPage, dpi: f32) -> Result<RgbImage, PdfError> {
    let config = PdfRenderConfig::new().scale_page_by_factor(dpi / POINTS_PER_INCH);
    let bitmap = page.render_with_config(&config)?;
    Ok(bitmap.as_image()?.to_rgb8())
}

#[allow(clippy::cast_precision_loss)] // pixel counts of a small picture
fn ink_ratio(image: &RgbImage) -> f64 {
    let ink = image.pixels().filter(|pixel| pixel.to_luma()[0] < INK_LUMA).count();
    ink as f64 / (f64::from(image.width()) * f64::from(image.height())).max(1.0)
}

pub(crate) fn save_new(document: &PdfDocument, target: &Path) -> Result<FileFacts, PdfError> {
    files::write_new(target, |temp| document.save_to_file(temp).map_err(PdfError::from))
}

fn degrees_of(rotation: PdfPageRenderRotation) -> i32 {
    match rotation {
        PdfPageRenderRotation::None => 0,
        PdfPageRenderRotation::Degrees90 => 90,
        PdfPageRenderRotation::Degrees180 => 180,
        PdfPageRenderRotation::Degrees270 => 270,
    }
}

fn rotation_of(degrees: i32) -> PdfPageRenderRotation {
    match degrees.rem_euclid(360) {
        90 => PdfPageRenderRotation::Degrees90,
        180 => PdfPageRenderRotation::Degrees180,
        270 => PdfPageRenderRotation::Degrees270,
        _ => PdfPageRenderRotation::None,
    }
}
