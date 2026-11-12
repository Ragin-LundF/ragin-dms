//! Synthetic PDFs and scans in temporary folders; never real documents.

use std::path::Path;

use image::{DynamicImage, Rgb, RgbImage};
use pdfium_render::prelude::{
    PdfColor, PdfPageImageObject, PdfPageObjectCommon, PdfPageObjectsCommon, PdfPagePaperSize, PdfPageTextObject,
    PdfPoints,
};

use crate::document::{page_of, render_rgb};
use crate::engine::PdfEngine;
use crate::fonts::{FontStyle, font_of};
use crate::rebuild::raster::deskew;
use crate::rebuild::{FontFamily, LayoutBox, LayoutLine};

const A4_WIDTH: f32 = 595.0;
const A4_HEIGHT: f32 = 842.0;
const TO_POINTS: f32 = 72.0 / 300.0;

#[allow(clippy::expect_used)] // tests need the library fetched by scripts/fetch-pdfium.sh
pub(crate) fn engine() -> PdfEngine {
    let library = PdfEngine::dev_library().expect("PDFium missing: run scripts/fetch-pdfium.sh");
    PdfEngine::bind(&library).expect("PDFium loads")
}

/// A PDF with one A4 page per text; `None` makes a page without any text, like a scan.
#[allow(clippy::expect_used)]
pub(crate) fn write_text_pdf(file: &Path, pages: &[Option<&str>]) {
    let mut document = engine().create().expect("new PDF");
    let font = document.fonts_mut().helvetica();
    for text in pages {
        let size = PdfPagePaperSize::Custom(PdfPoints::new(A4_WIDTH), PdfPoints::new(A4_HEIGHT));
        let mut page = document.pages_mut().create_page_at_end(size).expect("page");
        if let Some(text) = text {
            let mut object = PdfPageTextObject::new(&document, *text, font, PdfPoints::new(12.0)).expect("text");
            object.translate(PdfPoints::new(50.0), PdfPoints::new(700.0)).expect("place");
            page.objects_mut().add_text_object(object).expect("add text");
        }
    }
    std::fs::create_dir_all(file.parent().expect("parent")).expect("folder");
    document.save_to_file(file).expect("save");
}

/// A one-page text PDF carrying `bytes` as the file attachment `name`, like an e-invoice.
#[allow(clippy::expect_used)]
pub(crate) fn write_pdf_with_attachment(file: &Path, name: &str, bytes: &[u8]) {
    write_text_pdf(file, &[Some("Sample invoice")]);
    let mut document = engine().open(file).expect("reopen");
    document.attachments_mut().create_attachment_from_bytes(name, bytes).expect("attachment");
    let target = file.with_extension("attached.pdf");
    document.save_to_file(&target).expect("save");
    drop(document);
    std::fs::rename(&target, file).expect("replace");
}

/// A synthetic scan: a page image at 300 dpi with text in Liberation Sans 12 pt, and the PDF that holds
/// it as one picture, as a scanner writes it. Lines come back with boxes in 0..1000, as the AI answers.
pub(crate) struct TestScan {
    pub image: RgbImage,
}

impl TestScan {
    pub const WIDTH: u32 = 1600;
    pub const HEIGHT: u32 = 1200;

    pub fn new() -> Self {
        TestScan { image: RgbImage::from_pixel(Self::WIDTH, Self::HEIGHT, Rgb([255, 255, 255])) }
    }

    /// Draws [`text`] with its baseline at pixel [`x`], [`y`] in [`gray`], and returns the line as the AI
    /// would read it: [`read`] as text.
    #[allow(clippy::expect_used, clippy::cast_precision_loss)]
    pub fn text(&mut self, text: &str, (x, y): (u32, u32), gray: u8, legible: bool, read: &str) -> LayoutLine {
        let mut document = engine().create().expect("new PDF");
        let token = document.fonts_mut().load_true_type_from_bytes(font_of(FontStyle::SANS).bytes, true).expect("font");
        let size = PdfPagePaperSize::Custom(
            PdfPoints::new(Self::WIDTH as f32 * TO_POINTS),
            PdfPoints::new(Self::HEIGHT as f32 * TO_POINTS),
        );
        let mut page = document.pages_mut().create_page_at_end(size).expect("page");
        let mut object = PdfPageTextObject::new(&document, text, token, PdfPoints::new(12.0)).expect("text");
        object.set_fill_color(PdfColor::new(gray, gray, gray, 255)).expect("colour");
        object
            .translate(PdfPoints::new(x as f32 * TO_POINTS), PdfPoints::new((Self::HEIGHT - y) as f32 * TO_POINTS))
            .expect("place");
        page.objects_mut().add_text_object(object).expect("add text");
        let drawn = render_rgb(&page_of(&document, 0).expect("page"), 300.0).expect("render");
        let (mut left, mut top, mut right, mut bottom) = (u32::MAX, u32::MAX, 0, 0);
        for (px, py, pixel) in drawn.enumerate_pixels() {
            if pixel.0[0] < 250 {
                (left, top, right, bottom) = (left.min(px), top.min(py), right.max(px + 1), bottom.max(py + 1));
            }
            let target = self.image.get_pixel_mut(px, py);
            *target = Rgb([target.0[0].min(pixel.0[0]), target.0[1].min(pixel.0[1]), target.0[2].min(pixel.0[2])]);
        }
        let scale = |value: u32, size: u32| f64::from(value) * LayoutBox::SCALE / f64::from(size);
        LayoutLine {
            bounds: LayoutBox {
                left: scale(left, Self::WIDTH),
                top: scale(top, Self::HEIGHT),
                right: scale(right, Self::WIDTH),
                bottom: scale(bottom, Self::HEIGHT),
            },
            text: read.to_string(),
            bold: false,
            family: FontFamily::Sans,
            legible,
            kept: None,
        }
    }

    pub fn dark_text(&mut self, text: &str, at: (u32, u32)) -> LayoutLine {
        self.text(text, at, 0, true, text)
    }

    pub fn fill(&mut self, (x, y, width, height): (u32, u32, u32, u32), colour: [u8; 3]) {
        for py in y..y + height {
            for px in x..x + width {
                self.image.put_pixel(px, py, Rgb(colour));
            }
        }
    }

    /// The page turned by [`degrees`], clockwise, on white.
    pub fn turned(&self, degrees: f64) -> RgbImage {
        deskew::rotate(&self.image, degrees)
    }

    /// A PDF with [`page`] as its only content, at 300 dpi.
    #[allow(clippy::expect_used, clippy::cast_precision_loss)]
    pub fn write_pdf(file: &Path, page: &RgbImage) {
        let mut document = engine().create().expect("new PDF");
        let (width, height) = (page.width() as f32 * TO_POINTS, page.height() as f32 * TO_POINTS);
        let size = PdfPagePaperSize::Custom(PdfPoints::new(width), PdfPoints::new(height));
        let mut pdf_page = document.pages_mut().create_page_at_end(size).expect("page");
        let picture = DynamicImage::ImageRgb8(page.clone());
        let object =
            PdfPageImageObject::new_with_size(&document, &picture, PdfPoints::new(width), PdfPoints::new(height))
                .expect("image");
        pdf_page.objects_mut().add_image_object(object).expect("add image");
        document.save_to_file(file).expect("save");
    }
}
