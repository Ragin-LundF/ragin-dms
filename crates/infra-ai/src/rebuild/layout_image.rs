use std::io::Cursor;

use dms_application::port::ai::PageImage;
use image::imageops::{self, FilterType};
use image::{DynamicImage, ImageFormat};

/// The long side of a page sent for its layout.
const LAYOUT_SIDE: u32 = 1568;
/// The patch size of Qwen-VL models.
const PATCH: u32 = 28;

/// A page for reading its layout: at most [`LAYOUT_SIDE`] on the long side, both sides a multiple of 28, as lossless
/// PNG; with the width and height the prompt states. `None` if `page` is no readable picture.
pub(crate) fn layout_png(page: &PageImage) -> Option<(PageImage, u32, u32)> {
    let image = image::load_from_memory(&page.bytes).ok()?.to_rgb8();
    let scale = (f64::from(LAYOUT_SIDE) / f64::from(image.width().max(image.height()))).min(1.0);
    let width = on_patch_grid(f64::from(image.width()) * scale);
    let height = on_patch_grid(f64::from(image.height()) * scale);
    let resized = imageops::resize(&image, width, height, FilterType::CatmullRom);
    let mut bytes = Vec::new();
    DynamicImage::ImageRgb8(resized).write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png).ok()?;
    Some((PageImage { bytes, mime_type: "image/png".to_owned() }, width, height))
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // a positive side of at most 1568 pixels
fn on_patch_grid(side: f64) -> u32 {
    ((side / f64::from(PATCH)).round() as u32).max(1) * PATCH
}
