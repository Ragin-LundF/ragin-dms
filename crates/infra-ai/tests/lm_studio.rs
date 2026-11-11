//! Sends a synthetic invoice page to the local model the user runs. Opt-in only:
//! `DMS_AI_INTEGRATION=true cargo test -p dms-infra-ai --test lm_studio`, optionally with `DMS_AI_BASE_URL` and
//! `DMS_AI_MODEL`. Without the variable the test passes without doing anything.
#![allow(clippy::expect_used)]

use std::io::Cursor;
use std::sync::Arc;

use ab_glyph::{Font, FontRef, PxScale, ScaleFont, point};
use dms_application::port::ai::{LayoutOutcome, LineOutcome, PageImage, PageLayoutReader};
use dms_application::port::state::AiSettings;
use dms_domain::rebuild::LayoutBox;
use dms_infra_ai::AiAdapters;
use image::{DynamicImage, ImageFormat, Rgb, RgbImage};

const LINES: [&str; 12] = [
    "Beispiel Handwerk GmbH",
    "Musterstraße 12, 12345 Beispielstadt",
    "Rechnung Nr. 2026-0815",
    "Rechnungsdatum: 14.03.2026",
    "Herrn Jürgen Beispiel",
    "Gartenweg 7, 54321 Übungsdorf",
    "Pos. 1 Wartung der Heizungsanlage 240,00 €",
    "Pos. 2 Prüfung der Abgaswerte 85,50 €",
    "Zwischensumme 325,50 €",
    "Umsatzsteuer 19 % 61,85 €",
    "Gesamtbetrag 387,35 €",
    "Zahlbar innerhalb von 14 Tagen ohne Abzüge.",
];

#[test]
fn the_local_model_reads_most_lines_of_a_synthetic_invoice() {
    if std::env::var("DMS_AI_INTEGRATION").as_deref() != Ok("true") {
        return;
    }
    let settings = AiSettings {
        base_url: std::env::var("DMS_AI_BASE_URL").unwrap_or_else(|_| AiSettings::DEFAULT_BASE_URL.to_owned()),
        model: std::env::var("DMS_AI_MODEL").ok(),
        timeout_seconds: 900,
        ..AiSettings::default()
    };
    let reader = AiAdapters::new(Arc::new(move || settings.clone())).layout_reader;
    let page = invoice();

    let LayoutOutcome::Read(layout) = reader.read_layout(&png(&page)) else { panic!("the page was not read") };

    let lines: Vec<String> = layout
        .lines
        .iter()
        .map(|line| {
            // A NUL is a character lost under the JSON grammar; the rebuild reads such a line again from its crop.
            let text = if line.text.contains('\0') {
                match reader.read_line(&png(&crop_of(&page, line.bounds))) {
                    LineOutcome::Read(text) => text,
                    LineOutcome::Failure { .. } => line.text.clone(),
                }
            } else {
                line.text.clone()
            };
            text.split_whitespace().collect::<Vec<_>>().join(" ")
        })
        .collect();
    let found = LINES.iter().filter(|line| lines.iter().any(|read| read == *line)).count();
    assert!(found * 10 >= LINES.len() * 8, "found {found} of {}: {lines:?}", LINES.len());
}

/// An A4 page at 200 dpi with the synthetic invoice lines in Liberation Sans.
fn invoice() -> RgbImage {
    let font =
        FontRef::try_from_slice(include_bytes!("../../infra-pdf/fonts/LiberationSans-Regular.ttf")).expect("font");
    let scaled = font.as_scaled(PxScale::from(44.0));
    let mut page = RgbImage::from_pixel(1654, 2339, Rgb([255, 255, 255]));
    for (index, text) in LINES.iter().enumerate() {
        let mut x = 160.0;
        #[allow(clippy::cast_precision_loss)] // a dozen lines
        let baseline = 240.0 + index as f32 * 90.0;
        for character in text.chars() {
            let glyph = scaled.scaled_glyph(character);
            let advance = scaled.h_advance(glyph.id);
            if let Some(outline) = font.outline_glyph(ab_glyph::Glyph { position: point(x, baseline), ..glyph }) {
                let bounds = outline.px_bounds();
                outline.draw(|gx, gy, coverage| {
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // inside the page
                    let (px, py) =
                        ((bounds.min.x as i64 + i64::from(gx)) as u32, (bounds.min.y as i64 + i64::from(gy)) as u32);
                    if px < page.width() && py < page.height() {
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // coverage is 0..1
                        let ink = 255 - (coverage.clamp(0.0, 1.0) * 255.0) as u8;
                        let pixel = page.get_pixel_mut(px, py);
                        pixel.0 = pixel.0.map(|channel| channel.min(ink));
                    }
                });
            }
            x += advance;
        }
    }
    page
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // boxes are within 0..1000 of the page
fn crop_of(page: &RgbImage, bounds: LayoutBox) -> RgbImage {
    let pixels = |value: f64, size: u32| (value * f64::from(size) / LayoutBox::SCALE) as u32;
    let x = pixels(bounds.left, page.width()).min(page.width() - 1);
    let y = pixels(bounds.top, page.height()).min(page.height() - 1);
    let right = pixels(bounds.right, page.width()).clamp(x + 1, page.width());
    let bottom = pixels(bounds.bottom, page.height()).clamp(y + 1, page.height());
    image::imageops::crop_imm(page, x, y, right - x, bottom - y).to_image()
}

fn png(image: &RgbImage) -> PageImage {
    let mut bytes = Vec::new();
    DynamicImage::ImageRgb8(image.clone()).write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png).expect("png");
    PageImage { bytes, mime_type: "image/png".to_owned() }
}
