use std::collections::HashMap;
use std::io::Cursor;

use image::DynamicImage;
use pdfium_render::prelude::{
    PdfBitmap, PdfBitmapFormat, PdfColor, PdfDocument, PdfFontToken, PdfPageContentRegenerationStrategy,
    PdfPageImageObject, PdfPageObjectCommon, PdfPageObjectsCommon, PdfPagePaperSize, PdfPageTextObject, PdfPoints,
};

use super::crops::{Crop, CropImage};
use crate::error::PdfError;
use crate::fonts::{FontStyle, font_of};
use crate::images;
use crate::rebuild::fit::fitted_line::{FULL_SCALE, FittedLine, TextLine};

pub(crate) const DPI: f32 = 300.0;
const TO_POINTS: f32 = 72.0 / DPI;
const PHOTO_PIXELS: u64 = 300 * 300;
const SHADED: f64 = 0.4;

/// The bundled fonts loaded into one rebuilt document, each when a page first needs it. PDFium embeds
/// the whole font, not a subset.
#[derive(Default)]
pub(crate) struct DocumentFonts {
    tokens: HashMap<FontStyle, PdfFontToken>,
}

impl DocumentFonts {
    pub fn load(&mut self, document: &mut PdfDocument, style: FontStyle) -> Result<(), PdfError> {
        if let std::collections::hash_map::Entry::Vacant(entry) = self.tokens.entry(style) {
            entry.insert(document.fonts_mut().load_true_type_from_bytes(font_of(style).bytes, true)?);
        }
        Ok(())
    }

    fn token(&self, style: FontStyle) -> Result<PdfFontToken, PdfError> {
        self.tokens.get(&style).copied().ok_or_else(|| PdfError::InvalidRequest(format!("font {style:?} not loaded")))
    }
}

/// Writes one rebuilt page at the end of [`document`]: a white page of [`width`] x [`height`] pixels at
/// 300 dpi, the image crops at their exact place, then the real text on top. The fonts of the text lines
/// must be loaded.
#[allow(clippy::cast_precision_loss)] // page sizes are far below 2^24
pub(crate) fn add(
    document: &mut PdfDocument,
    fonts: &DocumentFonts,
    (width, height): (u32, u32),
    crops: &[Crop],
    lines: &[FittedLine],
) -> Result<(), PdfError> {
    let height_pt = height as f32 * TO_POINTS;
    let size = PdfPagePaperSize::Custom(PdfPoints::new(width as f32 * TO_POINTS), PdfPoints::new(height_pt));
    let mut page = document.pages_mut().create_page_at_end(size)?;
    page.set_content_regeneration_strategy(PdfPageContentRegenerationStrategy::Manual);
    for crop in crops {
        let mut object = image_object(document, crop)?;
        let bounds = crop.bounds;
        object.scale(bounds.width as f32 * TO_POINTS, bounds.height as f32 * TO_POINTS)?;
        object.translate(
            PdfPoints::new(bounds.x as f32 * TO_POINTS),
            PdfPoints::new(height_pt - bounds.bottom() as f32 * TO_POINTS),
        )?;
        page.objects_mut().add_image_object(object)?;
    }
    for line in lines {
        if let FittedLine::Text(text) = line {
            for object in text_objects(document, fonts, text, height_pt)? {
                page.objects_mut().add_text_object(object)?;
            }
        }
    }
    page.regenerate_content()?;
    Ok(())
}

#[allow(clippy::cast_precision_loss)] // pixel positions of a page
fn text_objects<'a>(
    document: &PdfDocument<'a>,
    fonts: &DocumentFonts,
    line: &TextLine,
    height_pt: f32,
) -> Result<Vec<PdfPageTextObject<'a>>, PdfError> {
    let token = fonts.token(line.style)?;
    let [red, green, blue] = line.color;
    let y = height_pt - line.baseline as f32 * TO_POINTS;
    let last = line.runs.len().saturating_sub(1);
    line.runs
        .iter()
        .enumerate()
        .map(|(i, run)| {
            // A trailing space keeps words apart when the text is copied.
            let shown = if i < last { format!("{} ", run.text) } else { run.text.clone() };
            let mut object = PdfPageTextObject::new(document, shown, token, PdfPoints::new(line.size_px * TO_POINTS))?;
            object.set_fill_color(PdfColor::new(red, green, blue, 255))?;
            object.transform(run.scale_percent / FULL_SCALE, 0.0, 0.0, 1.0, run.ink.x as f32 * TO_POINTS, y)?;
            Ok(object)
        })
        .collect()
}

/// Colour pictures and large shaded areas compress far better as JPEG; line art and text stay lossless,
/// grey crops as 8-bit grey.
fn image_object<'a>(document: &PdfDocument<'a>, crop: &Crop) -> Result<PdfPageImageObject<'a>, PdfError> {
    let large = crop.image.pixels() > PHOTO_PIXELS && (matches!(crop.image, CropImage::Rgb(_)) || crop.filled > SHADED);
    if large {
        let picture = match &crop.image {
            CropImage::Gray(image) => DynamicImage::ImageLuma8(image.clone()),
            CropImage::Rgb(image) => DynamicImage::ImageRgb8(image.clone()),
        };
        let jpeg = images::jpeg_bytes(&picture)?;
        return Ok(PdfPageImageObject::new_from_jpeg_reader(document, Cursor::new(jpeg))?);
    }
    let (width, height, format, mut buffer) = match &crop.image {
        CropImage::Gray(image) => {
            (image.width(), image.height(), PdfBitmapFormat::Gray, rows(image.width(), image.as_raw(), 1))
        }
        CropImage::Rgb(image) => {
            let bgr: Vec<u8> =
                image.as_raw().as_chunks::<3>().0.iter().flat_map(|&[red, green, blue]| [blue, green, red]).collect();
            (image.width(), image.height(), PdfBitmapFormat::BGR, rows(image.width(), &bgr, 3))
        }
    };
    let width = i32::try_from(width).map_err(PdfError::unsupported)?;
    let height = i32::try_from(height).map_err(PdfError::unsupported)?;
    let bitmap = PdfBitmap::from_bytes(width, height, format, &mut buffer)?;
    // shortcut: pdfium-render only makes an image object from a picture (RGBA with a soft mask), so a 1x1
    // placeholder is replaced by the grey or RGB bitmap; its soft mask stays as an unused ~100-byte object.
    // Upgrade when pdfium-render can create an empty image object with the image feature on.
    let mut object = PdfPageImageObject::new(document, &DynamicImage::new_luma8(1, 1))?;
    object.set_bitmap(&bitmap)?;
    Ok(object)
}

/// [`samples`] with each row padded to a multiple of 4 bytes, as PDFium bitmaps are laid out.
fn rows(width: u32, samples: &[u8], bytes_per_pixel: usize) -> Vec<u8> {
    let row = width as usize * bytes_per_pixel;
    let stride = row.div_ceil(4) * 4;
    if row == 0 || stride == row {
        return samples.to_vec();
    }
    samples
        .chunks_exact(row)
        .flat_map(|line| line.iter().copied().chain(std::iter::repeat_n(0, stride - row)))
        .collect()
}
