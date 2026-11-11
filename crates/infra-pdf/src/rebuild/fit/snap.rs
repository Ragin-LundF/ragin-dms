use std::ops::RangeInclusive;

use crate::rebuild::raster::mask::Mask;
use crate::rebuild::rect::Rect;

const MIN_HEIGHT: i32 = 8;
const MARGIN_SHARE: i32 = 100;
const DARK_SHARE: usize = 200;
const MAX_GAP: i32 = 2;
const NEAR_PERCENT: i32 = 35;
const THIN_TENTHS: i32 = 6;
const INSIDE_TENTHS: i32 = 3;
const MIN_HEIGHT_RATIO: f64 = 0.4;
const MAX_HEIGHT_RATIO: f64 = 2.5;
const MAX_CENTRE_SHIFT: f64 = 0.8;

type Band = RangeInclusive<i32>;

/// Moves the AI's box of a line onto the glyph ink near it: the band of inked rows that overlaps the box
/// most, widened by accents, i-dots and descenders, narrowed to the inked columns; `None` if there is no
/// ink or it is far off. The AI's numbers are only a hint where to look.
// Page coordinates fit i32, and they are clipped to the mask before they index it.
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap, clippy::cast_sign_loss)]
pub(crate) fn snap(glyph: &Mask, predicted: Rect) -> Option<Rect> {
    let (width, height_px) = (glyph.width as i32, glyph.height as i32);
    let height = predicted.height.max(MIN_HEIGHT);
    let x0 = (predicted.x - height / 2 - width / MARGIN_SHARE).max(0);
    let x1 = (predicted.right() + height / 2 + width / MARGIN_SHARE).min(width);
    let y0 = (predicted.y - height * 3 / 2).max(0);
    let y1 = (predicted.bottom() + height * 3 / 2).min(height_px);
    let columns = x0..x1;
    let bands = bands(glyph, &columns, y0..y1);
    // The first band with the most overlap, as Kotlin's `maxByOrNull` (Rust's `max_by_key` takes the last).
    let best =
        bands.iter().rev().max_by_key(|band| common(band, predicted)).filter(|band| common(band, predicted) > 0)?;
    let rows = widen(best, &bands, predicted);
    let inked: Vec<i32> = columns.filter(|&x| rows.clone().any(|y| glyph.get(x as usize, y as usize))).collect();
    let (&first, &last) = (inked.first()?, inked.last()?);
    let found = Rect::new(first, *rows.start(), last - first + 1, rows.end() - rows.start() + 1);
    let height_ratio = f64::from(found.height) / f64::from(predicted.height);
    let centre_shift = (found.center_y() - predicted.center_y()).abs() / f64::from(predicted.height);
    let plausible = (MIN_HEIGHT_RATIO..=MAX_HEIGHT_RATIO).contains(&height_ratio) && centre_shift < MAX_CENTRE_SHIFT;
    plausible.then_some(found)
}

/// Runs of rows with enough ink, allowing gaps of up to two rows.
#[allow(clippy::cast_sign_loss)] // clipped to the mask
fn bands(glyph: &Mask, columns: &std::ops::Range<i32>, rows: std::ops::Range<i32>) -> Vec<Band> {
    let min_dark = 2.max(columns.len() / DARK_SHARE);
    let mut bands = Vec::new();
    let mut start: Option<i32> = None;
    let mut gap = 0;
    let last = rows.end - 1;
    for y in rows {
        let dark = columns.clone().filter(|&x| glyph.get(x as usize, y as usize)).count();
        if dark >= min_dark {
            start.get_or_insert(y);
            gap = 0;
        } else if start.is_some() {
            gap += 1;
        }
        if gap > MAX_GAP {
            if let Some(from) = start.take() {
                bands.push(from..=y - gap);
            }
            gap = 0;
        }
    }
    if let Some(from) = start {
        bands.push(from..=last - gap);
    }
    bands
}

/// Accents, i-dots and descenders can form their own thin bands just above or below the line.
fn widen(best: &Band, bands: &[Band], predicted: Rect) -> Band {
    let (mut top, mut bottom) = (*best.start(), *best.end());
    let mut by_distance: Vec<&Band> = bands.iter().collect();
    by_distance.sort_by_key(|band| (band.start() - best.start()).abs());
    for band in by_distance {
        let line_height = bottom - top + 1;
        let band_height = band.end() - band.start() + 1;
        let near = band.start() - bottom <= line_height * NEAR_PERCENT / 100
            && top - band.end() <= line_height * NEAR_PERCENT / 100;
        let thin = band_height < line_height * THIN_TENTHS / 10;
        let mostly_inside = common(band, predicted) >= band_height * INSIDE_TENTHS / 10;
        if band != best && near && (thin || mostly_inside) {
            top = top.min(*band.start());
            bottom = bottom.max(*band.end());
        }
    }
    top..=bottom
}

fn common(band: &Band, rect: Rect) -> i32 {
    (*band.end()).min(rect.bottom()) - (*band.start()).max(rect.y)
}
