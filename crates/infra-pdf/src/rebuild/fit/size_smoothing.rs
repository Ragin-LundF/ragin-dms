use super::fitted_line::{FULL_SCALE, FittedLine};
use crate::fonts::font_of;

const SMOOTHING: f32 = 0.12;
const MIN_SCALE: f32 = 90.0;
const MAX_SCALE: f32 = 110.0;

/// A letter uses few font sizes: each size becomes the median of all sizes on the page within ±12 %, and
/// then each run is scaled horizontally to its ink, by 90–110 %.
#[allow(clippy::cast_precision_loss)] // ink widths of a page
pub(crate) fn smooth(lines: Vec<FittedLine>) -> Vec<FittedLine> {
    let sizes: Vec<f32> = lines
        .iter()
        .filter_map(|line| match line {
            FittedLine::Text(text) => Some(text.size_px),
            FittedLine::Kept { .. } => None,
        })
        .collect();
    lines
        .into_iter()
        .map(|line| {
            let FittedLine::Text(mut text) = line else {
                return line;
            };
            let mut near: Vec<f32> =
                sizes.iter().copied().filter(|size| (size - text.size_px).abs() <= text.size_px * SMOOTHING).collect();
            near.sort_by(f32::total_cmp);
            let size = near[near.len() / 2];
            let font = font_of(text.style);
            for run in &mut text.runs {
                let natural = font.width_at_1(&run.text) * size;
                run.scale_percent = (FULL_SCALE * run.ink.width as f32 / natural).clamp(MIN_SCALE, MAX_SCALE);
            }
            text.size_px = size;
            FittedLine::Text(text)
        })
        .collect()
}
