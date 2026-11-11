use super::fitted_line::{FULL_SCALE, FittedLine, TextLine, TextRun};
use super::{baseline, ink_color, size_smoothing, snap, word_ink};
use crate::fonts::{FontStyle, font_of};
use crate::rebuild::model::{KeepReason, LayoutLine, PageLayout};
use crate::rebuild::raster::mask::Mask;
use crate::rebuild::raster::page_masks::PageMasks;
use crate::rebuild::rect::{PixelBoxes, Rect};

const MIN_AGREEMENT: f32 = 0.75;
const MAX_AGREEMENT: f32 = 1.33;
const COVER_SHARE: f64 = 0.2;
const MIN_MEASURED: usize = 3;
const SHORT_SIZE: f32 = 0.9;
const READING_ROW: i32 = 20;
const COVERING: [&str; 4] = ["stamp", "signature", "handwrit", "seal"];

/// Places the AI's lines on the page, measured on the ink rather than trusted from the AI: the box is
/// moved onto the glyph ink, the baseline is measured, words are matched to ink gaps, the size comes
/// from the ink width, and sizes are smoothed across the page. A line stays an image if the AI is not
/// sure, no ink is there, a stamp or handwriting covers it, the font cannot encode it, or the size from
/// the width disagrees with the height of the ink (missing or invented words).
pub(crate) fn fit(masks: &PageMasks, layout: &PageLayout, boxes: PixelBoxes) -> Vec<FittedLine> {
    let covers: Vec<Rect> = layout
        .graphics
        .iter()
        .filter(|graphic| {
            let kind = graphic.kind.to_lowercase();
            COVERING.iter().any(|covering| kind.contains(covering))
        })
        .map(|graphic| boxes.to_pixels(graphic.bounds))
        .collect();
    let mut fitted: Vec<FittedLine> =
        layout.lines.iter().map(|line| fit_with_retry(masks, line, boxes.to_pixels(line.bounds), &covers)).collect();
    fitted.sort_by_key(|line| (line.bounds().y / READING_ROW, line.bounds().x));
    size_smoothing::smooth(fitted)
}

/// Grey or coloured print is only partly darker than the ink threshold: it is measured on the faint
/// mask too.
fn fit_with_retry(masks: &PageMasks, line: &LayoutLine, predicted: Rect, covers: &[Rect]) -> FittedLine {
    let first = fit_one(masks, &masks.glyph_ink, line, predicted, covers);
    let FittedLine::Kept { reason: first_reason, .. } = first else {
        return first;
    };
    if first_reason != KeepReason::NoInk && first_reason != KeepReason::TextMismatch {
        return first;
    }
    let second = fit_one(masks, masks.faint_glyph_ink(), line, predicted, covers);
    let second_found_ink = !matches!(second, FittedLine::Kept { reason: KeepReason::NoInk, .. });
    match second {
        FittedLine::Text(_) => second,
        FittedLine::Kept { .. } if first_reason == KeepReason::NoInk && second_found_ink => second,
        FittedLine::Kept { .. } => first,
    }
}

fn fit_one(masks: &PageMasks, glyph: &Mask, line: &LayoutLine, predicted: Rect, covers: &[Rect]) -> FittedLine {
    let text = line.text.split_whitespace().collect::<Vec<_>>().join(" ");
    let known = line.kept.or_else(|| text.is_empty().then_some(KeepReason::NotSure));
    if let Some(reason) = known {
        return FittedLine::Kept { bounds: predicted, reason };
    }
    let Some(bounds) = snap::snap(glyph, predicted) else {
        return FittedLine::Kept { bounds: predicted, reason: KeepReason::NoInk };
    };
    let style = FontStyle { family: line.family, bold: line.bold };
    let reason = if !line.legible {
        KeepReason::NotSure
    } else if covers.iter().any(|cover| overlap(*cover, bounds) > COVER_SHARE) {
        KeepReason::Covered
    } else if !font_of(style).can_encode(&text) {
        KeepReason::NotEncodable
    } else {
        return measure(masks, glyph, &text, bounds, style);
    };
    FittedLine::Kept { bounds, reason }
}

#[allow(clippy::cast_precision_loss)] // pixel sizes of a page
fn measure(masks: &PageMasks, glyph: &Mask, text: &str, bounds: Rect, style: FontStyle) -> FittedLine {
    let font = font_of(style);
    let baseline = baseline::of(glyph, bounds);
    let words: Vec<&str> = text.split(' ').collect();
    let runs: Vec<TextRun> = match word_ink::of(glyph, bounds, baseline, words.len()) {
        Some(inks) => words
            .iter()
            .zip(inks)
            .map(|(word, ink)| TextRun { text: (*word).to_string(), ink, scale_percent: FULL_SCALE })
            .collect(),
        None => vec![TextRun {
            text: text.to_string(),
            ink: word_ink::tighten(glyph, bounds, baseline),
            scale_percent: FULL_SCALE,
        }],
    };
    let ink_width: f32 = runs.iter().map(|run| run.ink.width as f32).sum();
    let length = text.chars().count();
    let size = if length < MIN_MEASURED {
        bounds.height as f32 * SHORT_SIZE
    } else {
        ink_width / runs.iter().map(|run| font.width_at_1(&run.text)).sum::<f32>()
    };
    let size_from_height = (baseline - bounds.y) as f32 / font.ink_top(text);
    if length >= MIN_MEASURED && !(MIN_AGREEMENT..=MAX_AGREEMENT).contains(&(size / size_from_height)) {
        return FittedLine::Kept { bounds, reason: KeepReason::TextMismatch };
    }
    FittedLine::Text(TextLine {
        bounds,
        style,
        runs,
        baseline,
        size_px: size,
        color: ink_color::of(masks, glyph, bounds),
    })
}

/// The share of [`bounds`] that [`cover`] hides.
fn overlap(cover: Rect, bounds: Rect) -> f64 {
    let common = cover.intersection(bounds);
    if common.is_empty() { 0.0 } else { common.area() / bounds.area() }
}
