//! "Convert to real PDF": a scanned PDF rebuilt as a new PDF with real text where the scan could be read
//! and verified, and the original pixels everywhere else.

mod compose;
mod fit;
mod model;
pub(crate) mod raster;
mod rect;

use std::fs;
use std::path::{Path, PathBuf};

use image::{DynamicImage, RgbImage};

pub use model::{
    FontFamily, KeepReason, LayoutBox, LayoutGraphic, LayoutLine, PageInput, PageLayout, PreparedPage, RebuildRegion,
    RebuiltFile, RebuiltPage, RegionKind,
};

use self::compose::page_composer;
use self::compose::page_writer::{self, DocumentFonts};
use self::raster::deskew;
use self::rect::{PixelBoxes, Rect};
use crate::document::{page_of, pages_in, render_rgb, save_new};
use crate::engine::PdfEngine;
use crate::error::PdfError;
use crate::files::{self, FileFacts};
use crate::images::{self, PageImage};

const MIN_CROP_PAD: i32 = 6;
const CROP_PAD_TENTHS: i32 = 4;

/// The rebuild. Rebuilt PDFs wait in [`work_directory`], in the app's own data, until they are placed in
/// a library folder or discarded. Make it once when the app starts: that is when leftovers of an earlier
/// run are deleted. The source never changes.
pub struct Rebuilder {
    engine: PdfEngine,
    work_directory: PathBuf,
}

impl Rebuilder {
    #[must_use]
    pub fn new(engine: PdfEngine, work_directory: PathBuf) -> Self {
        let rebuilder = Rebuilder { engine, work_directory };
        rebuilder.discard_all();
        rebuilder
    }

    /// Page [`page`] of [`path`] rendered at 300 dpi and straightened.
    ///
    /// # Errors
    /// [`PdfError::NotFound`] for a missing file or page; [`PdfError::UnsupportedFormat`] if it cannot be
    /// opened or rendered.
    pub fn prepare_page(&self, path: &Path, page: usize) -> Result<PreparedPage, PdfError> {
        let document = self.engine.open(path)?;
        let rendered = render_rgb(&page_of(&document, page).ok_or(PdfError::NotFound)?, page_writer::DPI)?;
        let angle = deskew::angle_of(&rendered);
        let straight = if angle == 0.0 { rendered } else { deskew::rotate(&rendered, -angle) };
        Ok(PreparedPage { page_index: page, image: images::png_of(&DynamicImage::ImageRgb8(straight))?, angle })
    }

    /// The part of [`prepared`] around [`bounds`], for reading one line again, as PNG.
    ///
    /// # Errors
    /// [`PdfError::UnsupportedFormat`] if the prepared image is not readable.
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)] // page sides fit i32, the area is clipped
    pub fn crop(&self, prepared: &PreparedPage, bounds: LayoutBox) -> Result<PageImage, PdfError> {
        let page = images::decode(&prepared.image)?.to_rgb8();
        let (width, height) = (page.width() as i32, page.height() as i32);
        let line = PixelBoxes { width, height }.to_pixels(bounds);
        let pad = MIN_CROP_PAD.max(line.height * CROP_PAD_TENTHS / 10);
        let area = Rect::new(line.x - pad, line.y - pad, line.width + 2 * pad, line.height + 2 * pad)
            .intersection(Rect::new(0, 0, width, height));
        let cut = image::imageops::crop_imm(&page, area.x as u32, area.y as u32, area.width as u32, area.height as u32);
        images::png_of(&DynamicImage::ImageRgb8(cut.to_image()))
    }

    /// Writes the rebuilt PDF of [`source`], whose content is [`source_hash`], from [`pages`], in their
    /// order.
    ///
    /// # Errors
    /// [`PdfError::NotFound`] for a missing source or page; [`PdfError::UnsupportedFormat`] if a page
    /// cannot be read or written.
    pub fn write(&self, source: &Path, source_hash: &str, pages: &[PageInput]) -> Result<RebuiltFile, PdfError> {
        let original = self.engine.open(source)?;
        let mut rebuilt = self.engine.create()?;
        let mut fonts = DocumentFonts::default();
        let mut summaries = Vec::with_capacity(pages.len());
        for page in pages {
            let summary = match page {
                PageInput::Failed { page_index } => {
                    if *page_index >= pages_in(&original) {
                        return Err(PdfError::NotFound);
                    }
                    let index = i32::try_from(*page_index).map_err(|_| PdfError::NotFound)?;
                    let end = rebuilt.pages().len();
                    rebuilt.pages_mut().copy_page_from_document(&original, index, end)?;
                    page_composer::failed()
                }
                PageInput::Read { prepared, layout } => {
                    let image: RgbImage = images::decode(&prepared.image)?.to_rgb8();
                    page_composer::add(&mut rebuilt, &mut fonts, &image, prepared.angle, layout)?
                }
            };
            summaries.push(summary);
        }
        let id = uuid::Uuid::new_v4().to_string();
        let facts = save_new(&rebuilt, &self.file_of(&id)?)?;
        Ok(RebuiltFile { id, pages: summaries, size: facts.size, source_hash: source_hash.to_string() })
    }

    /// One page of [`rebuilt`] as a PNG picture; `Ok(None)` if the page does not exist.
    ///
    /// # Errors
    /// The errors of [`PdfEngine::render_page`]; [`PdfError::InvalidRequest`] for an id that is not ours.
    pub fn render_page(
        &self,
        rebuilt: &RebuiltFile,
        page: usize,
        max_side: u32,
    ) -> Result<Option<PageImage>, PdfError> {
        self.engine.render_page(&self.file_of(&rebuilt.id)?, page, max_side)
    }

    /// Copies [`rebuilt`] to [`target`], which must not exist yet.
    ///
    /// # Errors
    /// [`PdfError::AlreadyExists`] if [`target`] exists; [`PdfError::Io`] if the copy fails.
    pub fn place(&self, rebuilt: &RebuiltFile, target: &Path) -> Result<FileFacts, PdfError> {
        let file = self.file_of(&rebuilt.id)?;
        files::write_new(target, |temp| {
            fs::copy(&file, temp)?;
            Ok(())
        })
    }

    /// Deletes [`rebuilt`]; it was never part of the library.
    pub fn discard(&self, rebuilt: &RebuiltFile) {
        if let Ok(file) = self.file_of(&rebuilt.id) {
            let _ = fs::remove_file(file);
        }
    }

    /// Rebuilds left over from an earlier run were never chosen; cleaning up never keeps the app from
    /// starting, and what cannot be deleted waits for the next start.
    fn discard_all(&self) {
        if let Ok(entries) = fs::read_dir(&self.work_directory) {
            for entry in entries.flatten() {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    /// The file of a rebuilt PDF; [`id`] must be a UUID, so it can never point outside the work folder.
    fn file_of(&self, id: &str) -> Result<PathBuf, PdfError> {
        let id = uuid::Uuid::parse_str(id).map_err(|_| PdfError::InvalidRequest("not a rebuild id".into()))?;
        Ok(self.work_directory.join(format!("{}.pdf", id.hyphenated())))
    }
}

#[cfg(test)]
mod tests;
