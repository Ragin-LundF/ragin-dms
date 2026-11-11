use std::path::Path;

use pdfium_render::prelude::{
    PdfPageContentRegenerationStrategy, PdfPageObjectsCommon, PdfPageTextObject, PdfPageTextRenderMode, PdfPoints,
};

use super::pages::{page_of, page_text, pages_in, save_new};
use crate::engine::PdfEngine;
use crate::error::PdfError;
use crate::files::FileFacts;
use crate::fonts::{FontStyle, font_of};

/// A scan may carry a page number or a stamp as text; less than this per page is no text layer.
const MIN_CHARACTERS_PER_PAGE: usize = 20;
const FONT_SIZE: f32 = 8.0;
const LEADING: f32 = 10.0;
const MARGIN: f32 = 36.0;

impl PdfEngine {
    /// Whether the PDF has a usable text layer on its pages, as a scan without OCR has not: at least
    /// 20 non-whitespace characters per page on average.
    ///
    /// # Errors
    /// [`PdfError::NotFound`] or [`PdfError::UnsupportedFormat`] if the file cannot be opened.
    pub fn has_text_layer(self, path: &Path) -> Result<bool, PdfError> {
        let document = self.open(path)?;
        let pages = pages_in(&document);
        if pages == 0 {
            return Ok(false);
        }
        let needed = MIN_CHARACTERS_PER_PAGE * pages;
        let mut found = 0;
        for page in document.pages().iter() {
            found += page_text(&page)?.chars().filter(|character| !character.is_whitespace()).count();
            if found >= needed {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// A copy of [`source`] with one text per page as invisible text (render mode "invisible", as OCR
    /// programs do) in the embedded Liberation Sans, so a scan can be searched and its text copied.
    /// Without word positions the text runs line by line from the top of the page; texts beyond the
    /// last page go on the last page, and characters the font lacks are left out.
    ///
    /// # Errors
    /// [`PdfError::AlreadyExists`] if [`target`] exists; the errors of opening [`source`].
    pub fn add_text_layer(self, source: &Path, pages: &[String], target: &Path) -> Result<FileFacts, PdfError> {
        let mut document = self.open(source)?;
        let font = font_of(FontStyle::SANS);
        let token = document.fonts_mut().load_true_type_from_bytes(font.bytes, true)?;
        let texts = spread(pages, pages_in(&document));
        for (index, text) in texts.iter().enumerate().filter(|(_, text)| !text.trim().is_empty()) {
            let mut page = page_of(&document, index).ok_or(PdfError::NotFound)?;
            page.set_content_regeneration_strategy(PdfPageContentRegenerationStrategy::Manual);
            let media = page.boundaries().media()?.bounds;
            let left = media.left().value + MARGIN;
            let mut baseline = media.top().value - MARGIN;
            for line in text.lines() {
                let shown: String = line.chars().filter(|c| !c.is_control() && font.has_glyph(*c)).collect();
                if !shown.is_empty() {
                    let mut object = PdfPageTextObject::new(&document, shown, token, PdfPoints::new(FONT_SIZE))?;
                    object.set_render_mode(PdfPageTextRenderMode::Invisible)?;
                    object.translate(PdfPoints::new(left), PdfPoints::new(baseline))?;
                    page.objects_mut().add_text_object(object)?;
                }
                baseline -= LEADING;
            }
            page.regenerate_content()?;
        }
        save_new(&document, target)
    }
}

/// One text per page; extra texts are joined onto the last page.
fn spread(pages: &[String], count: usize) -> Vec<String> {
    if pages.len() <= count || count == 0 {
        return pages.to_vec();
    }
    let mut texts = pages[..count - 1].to_vec();
    texts.push(pages[count - 1..].join("\n"));
    texts
}

#[cfg(test)]
mod tests {
    use super::spread;

    #[test]
    fn texts_beyond_the_last_page_are_joined_onto_it() {
        let texts = ["one", "two", "three"].map(String::from);

        assert_eq!(spread(&texts, 2), vec!["one".to_string(), "two\nthree".to_string()]);
        assert_eq!(spread(&texts[..1], 2), vec!["one".to_string()]);
    }
}
