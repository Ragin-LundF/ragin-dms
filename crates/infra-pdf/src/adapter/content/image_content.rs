//! A photo or scanned image, downscaled for a vision model or for a thumbnail.

use std::path::Path;

use dms_application::port::ai::{DocumentContent, PageImage};
use image::{DynamicImage, ImageError, ImageReader};

use crate::adapter::convert;
use crate::images::{VISION_MAX_SIDE, thumbnail_jpeg, vision_jpeg};

const NOT_SUPPORTED: &str = "The image format is not supported";

pub(crate) fn read(file: &Path) -> DocumentContent {
    let picture = match decode(file) {
        Ok(image) => vision_jpeg(&image, VISION_MAX_SIDE),
        Err(ImageError::IoError(error)) => return DocumentContent::NotReadable(error.to_string()),
        Err(_) => return DocumentContent::NotReadable(NOT_SUPPORTED.to_owned()),
    };
    match picture {
        Ok(picture) => DocumentContent::PageImages(vec![convert::page_image(picture)]),
        Err(_) => DocumentContent::NotReadable(NOT_SUPPORTED.to_owned()),
    }
}

/// The image at most `max_side` pixels on its long side; `None` for a file that is no readable image.
pub(crate) fn thumbnail(file: &Path, max_side: u32) -> Option<PageImage> {
    thumbnail_jpeg(&decode(file).ok()?, max_side).ok().map(convert::page_image)
}

/// The picture in `file`, recognized by its content rather than its extension, as Java's `ImageIO` did.
fn decode(file: &Path) -> Result<DynamicImage, ImageError> {
    ImageReader::open(file)?.with_guessed_format()?.decode()
}
