use std::io::Cursor;

use image::codecs::jpeg::JpegEncoder;
use image::imageops::{self, FilterType};
use image::{DynamicImage, ImageFormat, RgbImage};

use crate::error::PdfError;

/// An encoded picture that crosses the port: PNG or JPEG bytes with their MIME type.
#[derive(Clone, PartialEq, Eq)]
pub struct PageImage {
    pub bytes: Vec<u8>,
    pub mime_type: &'static str,
}

impl std::fmt::Debug for PageImage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "PageImage({}, {} bytes)", self.mime_type, self.bytes.len())
    }
}

pub(crate) const PNG: &str = "image/png";
pub(crate) const JPEG: &str = "image/jpeg";
pub(crate) const JPEG_QUALITY: u8 = 85;

/// The long side of a picture for a vision model.
pub const VISION_MAX_SIDE: u32 = 1600;
/// The long side of a page sent for its layout.
pub const LAYOUT_SIDE: u32 = 1568;
/// The patch size of Qwen-VL models.
const PATCH: u32 = 28;

/// Lossless, so the rebuild masks see the pixels the AI saw.
pub(crate) fn png_of(image: &DynamicImage) -> Result<PageImage, PdfError> {
    let mut bytes = Vec::new();
    image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png).map_err(PdfError::unsupported)?;
    Ok(PageImage { bytes, mime_type: PNG })
}

pub(crate) fn jpeg_bytes(image: &DynamicImage) -> Result<Vec<u8>, PdfError> {
    let mut bytes = Vec::new();
    JpegEncoder::new_with_quality(&mut bytes, JPEG_QUALITY).encode_image(image).map_err(PdfError::unsupported)?;
    Ok(bytes)
}

pub(crate) fn decode(image: &PageImage) -> Result<DynamicImage, PdfError> {
    image::load_from_memory(&image.bytes).map_err(|_| PdfError::unsupported("not a readable page image"))
}

/// A picture for a vision model or the screen: at most [`max_side`] pixels on the long side, never
/// enlarged, as JPEG (transparent parts become white). `VisionImages.of` in the Kotlin app.
///
/// # Errors
/// [`PdfError::UnsupportedFormat`] if the picture cannot be encoded.
pub fn vision_jpeg(image: &DynamicImage, max_side: u32) -> Result<PageImage, PdfError> {
    let rgb = on_white(image);
    let scaled = scaled_down(&rgb, max_side);
    Ok(PageImage { bytes: jpeg_bytes(&DynamicImage::ImageRgb8(scaled))?, mime_type: JPEG })
}

/// A page for reading its layout: at most [`LAYOUT_SIDE`] on the long side, both sides a multiple of 28
/// (the patch size of Qwen-VL models), as lossless PNG. `VisionImages.layoutPng` in the Kotlin app; the
/// returned width and height are what the prompt states.
///
/// # Errors
/// [`PdfError::UnsupportedFormat`] if [`page`] is no readable picture.
pub fn layout_png(page: &PageImage) -> Result<(PageImage, u32, u32), PdfError> {
    let image = decode(page)?.to_rgb8();
    let scale = (f64::from(LAYOUT_SIDE) / f64::from(image.width().max(image.height()))).min(1.0);
    let width = on_patch_grid(f64::from(image.width()) * scale);
    let height = on_patch_grid(f64::from(image.height()) * scale);
    let resized = imageops::resize(&image, width, height, FilterType::CatmullRom);
    Ok((png_of(&DynamicImage::ImageRgb8(resized))?, width, height))
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // a positive side of at most 1568 pixels
fn on_patch_grid(side: f64) -> u32 {
    ((side / f64::from(PATCH)).round() as u32).max(1) * PATCH
}

/// [`image`] at most [`max_side`] pixels on its long side, never enlarged.
pub(crate) fn scaled_down(image: &RgbImage, max_side: u32) -> RgbImage {
    let longest = image.width().max(image.height());
    if longest <= max_side {
        return image.clone();
    }
    let factor = f64::from(max_side) / f64::from(longest);
    let width = scaled_side(image.width(), factor);
    let height = scaled_side(image.height(), factor);
    imageops::resize(image, width, height, FilterType::Triangle)
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // a positive pixel count below the source side
fn scaled_side(side: u32, factor: f64) -> u32 {
    ((f64::from(side) * factor).round() as u32).max(1)
}

fn on_white(image: &DynamicImage) -> RgbImage {
    let rgba = image.to_rgba8();
    RgbImage::from_fn(rgba.width(), rgba.height(), |x, y| {
        let [red, green, blue, alpha] = rgba.get_pixel(x, y).0;
        let blend = |channel: u8| {
            let value = (u32::from(channel) * u32::from(alpha) + 255 * (255 - u32::from(alpha))) / 255;
            u8::try_from(value).unwrap_or(u8::MAX)
        };
        image::Rgb([blend(red), blend(green), blend(blue)])
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_layout_page_is_at_most_1568_pixels_and_on_the_patch_grid() {
        let page = png_of(&DynamicImage::ImageRgb8(RgbImage::new(2480, 3508))).expect("png");

        let (sent, width, height) = layout_png(&page).expect("layout png");

        assert_eq!((width, height), (1120, 1568));
        assert_eq!(sent.mime_type, PNG);
        assert_eq!(decode(&sent).expect("decode").width(), 1120);
    }

    #[test]
    fn a_small_layout_page_is_not_enlarged_beyond_the_grid() {
        let page = png_of(&DynamicImage::ImageRgb8(RgbImage::new(100, 10))).expect("png");

        let (_, width, height) = layout_png(&page).expect("layout png");

        assert_eq!((width, height), (112, 28));
    }

    #[test]
    fn a_vision_picture_is_a_jpeg_scaled_to_its_long_side_and_transparency_is_white() {
        let transparent = DynamicImage::ImageRgba8(image::RgbaImage::new(3200, 1000));

        let picture = vision_jpeg(&transparent, VISION_MAX_SIDE).expect("jpeg");

        assert_eq!(picture.mime_type, JPEG);
        let decoded = decode(&picture).expect("decode").to_rgb8();
        assert_eq!((decoded.width(), decoded.height()), (1600, 500));
        assert!(decoded.get_pixel(10, 10).0.iter().all(|&channel| channel > 250));
    }
}
