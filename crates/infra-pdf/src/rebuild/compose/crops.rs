use image::{GrayImage, Luma, Rgb, RgbImage};

use crate::rebuild::fit::fitted_line::FittedLine;
use crate::rebuild::raster::labels::Labels;
use crate::rebuild::raster::mask::Mask;
use crate::rebuild::raster::page_masks::PageMasks;
use crate::rebuild::rect::Rect;

const DUST: usize = 8;
const GROUP: usize = 6;
const TEXT_PAD: f64 = 0.3;
const TEXT_SIDE: i32 = 2;
const COLOUR_SPREAD: u8 = 24;
const WHITE: u8 = 255;

/// A piece of the scan that stays an image: a logo, stamp, signature, rule, or a line that was not
/// replaced. [`filled`] is the share of the box that holds scan pixels; large filled crops are shaded
/// areas or photos, not line art.
pub(crate) struct Crop {
    pub bounds: Rect,
    pub image: CropImage,
    pub filled: f64,
}

pub(crate) enum CropImage {
    Gray(GrayImage),
    Rgb(RgbImage),
}

impl CropImage {
    pub fn pixels(&self) -> u64 {
        let (width, height) = match self {
            CropImage::Gray(image) => image.dimensions(),
            CropImage::Rgb(image) => image.dimensions(),
        };
        u64::from(width) * u64::from(height)
    }
}

/// Cuts everything that is not replaced text out of the scan. A component of [`PageMasks::light`]
/// becomes graphics if less than half of it lies in the text areas; rules always do. Graphics within a
/// few pixels of each other form one crop, which keeps the original pixels on and next to the graphics
/// and white elsewhere, so thin strokes keep their soft edges.
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)] // page sizes fit i32
pub(crate) fn of(masks: &PageMasks, lines: &[FittedLine]) -> Vec<Crop> {
    let run_boxes: Vec<Rect> = lines
        .iter()
        .filter_map(|line| match line {
            FittedLine::Text(text) => Some(text),
            FittedLine::Kept { .. } => None,
        })
        .flat_map(|line| {
            line.runs.iter().map(|run| Rect::new(run.ink.x, line.bounds.y, run.ink.width, line.bounds.height))
        })
        .collect();
    let kept_boxes: Vec<Rect> =
        lines.iter().filter(|line| matches!(line, FittedLine::Kept { .. })).map(FittedLine::bounds).collect();
    let graphics = graphics_of(masks, &run_boxes, &kept_boxes);
    let keep = graphics.dilate(1);
    let page = Rect::new(0, 0, masks.width as i32, masks.height as i32);
    Labels::of(&graphics.dilate(GROUP))
        .boxes
        .into_iter()
        .map(|group| {
            let bounds = group.intersection(page);
            Crop { bounds, image: cut(masks, bounds, &keep), filled: filled_share(&keep, bounds) }
        })
        .collect()
}

#[allow(clippy::cast_possible_truncation)] // a padding of a few pixels
fn graphics_of(masks: &PageMasks, run_boxes: &[Rect], kept_boxes: &[Rect]) -> Mask {
    let mut text_areas = Mask::new(masks.width, masks.height);
    for run in run_boxes {
        let pad = (f64::from(run.height) * TEXT_PAD) as i32;
        text_areas.fill(Rect::new(run.x - TEXT_SIDE, run.y - pad, run.width + 2 * TEXT_SIDE, run.height + 2 * pad));
    }
    let labels = Labels::of(&masks.light);
    let mut inside = vec![0_usize; labels.len()];
    for (i, id) in labels.ids.iter().enumerate() {
        if let (Some(id), true) = (id, text_areas.bits[i]) {
            inside[*id as usize] += 1;
        }
    }
    let is_graphic: Vec<bool> = (0..labels.len())
        .map(|id| {
            let dust = labels.counts[id] < DUST && !kept_boxes.iter().any(|kept| kept.intersects(labels.boxes[id]));
            !dust && inside[id] * 2 < labels.counts[id]
        })
        .collect();
    let bits = labels
        .ids
        .iter()
        .zip(&masks.rules.bits)
        .map(|(id, &rule)| id.is_some_and(|id| is_graphic[id as usize]) || rule)
        .collect();
    let mut graphics = Mask { width: masks.width, height: masks.height, bits };
    // Letters touching a table rule must not end up twice: inside replaced words only the rule stays.
    for run in run_boxes {
        let (columns, rows) = graphics.clipped(*run);
        for y in rows {
            for x in columns.clone() {
                if !masks.rules.get(x, y) {
                    graphics.set(x, y, false);
                }
            }
        }
    }
    graphics
}

#[allow(clippy::cast_precision_loss)] // pixel counts of a page
fn filled_share(keep: &Mask, bounds: Rect) -> f64 {
    let (columns, rows) = keep.clipped(bounds);
    let kept = rows.flat_map(|y| columns.clone().map(move |x| (x, y))).filter(|&(x, y)| keep.get(x, y)).count();
    kept as f64 / bounds.area()
}

#[allow(clippy::cast_possible_truncation)] // crop sides are page sides
fn cut(masks: &PageMasks, bounds: Rect, keep: &Mask) -> CropImage {
    let (columns, rows) = keep.clipped(bounds);
    let (left, top) = (columns.start, rows.start);
    let (width, height) = (columns.len() as u32, rows.len() as u32);
    let index = |x: u32, y: u32| (top + y as usize) * masks.width + left + x as usize;
    let kept = |x: u32, y: u32| keep.get(left + x as usize, top + y as usize);
    if is_colourful(masks, bounds, keep) {
        CropImage::Rgb(RgbImage::from_fn(width, height, |x, y| {
            if kept(x, y) { Rgb(masks.rgb(index(x, y))) } else { Rgb([WHITE; 3]) }
        }))
    } else {
        CropImage::Gray(GrayImage::from_fn(width, height, |x, y| {
            Luma([if kept(x, y) { masks.gray[index(x, y)] } else { WHITE }])
        }))
    }
}

fn is_colourful(masks: &PageMasks, bounds: Rect, keep: &Mask) -> bool {
    let (columns, rows) = keep.clipped(bounds);
    rows.flat_map(|y| columns.clone().map(move |x| y * masks.width + x))
        .any(|i| keep.bits[i] && spread(masks.rgb(i)) > COLOUR_SPREAD)
}

fn spread([red, green, blue]: [u8; 3]) -> u8 {
    red.abs_diff(green).max(green.abs_diff(blue)).max(red.abs_diff(blue))
}
