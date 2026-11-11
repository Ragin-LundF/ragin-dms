use std::collections::BTreeMap;

use image::RgbImage;
use pdfium_render::prelude::PdfDocument;

use super::crops;
use super::page_writer::{self, DocumentFonts};
use crate::error::PdfError;
use crate::rebuild::fit::fitted_line::FittedLine;
use crate::rebuild::fit::line_fitter;
use crate::rebuild::model::{KeepReason, LayoutBox, PageLayout, RebuildRegion, RebuiltPage, RegionKind};
use crate::rebuild::raster::page_masks::PageMasks;
use crate::rebuild::rect::PixelBoxes;

/// Rebuilds one straightened page from its layout: masks, fitted lines, crops, and the written page.
#[allow(clippy::cast_possible_wrap)] // page sides fit i32
pub(crate) fn add(
    document: &mut PdfDocument,
    fonts: &mut DocumentFonts,
    page: &RgbImage,
    angle: f64,
    layout: &PageLayout,
) -> Result<RebuiltPage, PdfError> {
    let masks = PageMasks::new(page);
    let boxes = PixelBoxes { width: page.width() as i32, height: page.height() as i32 };
    let lines = line_fitter::fit(&masks, layout, boxes);
    let crops = crops::of(&masks, &lines);
    for line in &lines {
        if let FittedLine::Text(text) = line {
            fonts.load(document, text.style)?;
        }
    }
    page_writer::add(document, fonts, page.dimensions(), &crops, &lines)?;
    let regions = lines
        .iter()
        .map(|line| {
            let kind = if matches!(line, FittedLine::Text(_)) { RegionKind::Text } else { RegionKind::KeptLine };
            RebuildRegion { bounds: boxes.to_layout(line.bounds()), kind }
        })
        .chain(crops.iter().map(|crop| RebuildRegion { bounds: boxes.to_layout(crop.bounds), kind: RegionKind::Image }))
        .collect();
    let mut kept_reasons = BTreeMap::new();
    for line in &lines {
        if let FittedLine::Kept { reason, .. } = line {
            *kept_reasons.entry(*reason).or_insert(0) += 1;
        }
    }
    let kept: usize = kept_reasons.values().sum();
    Ok(RebuiltPage { lines_as_text: lines.len() - kept, lines_total: lines.len(), angle, kept_reasons, regions })
}

/// A page the AI could not read, kept as it was.
pub(crate) fn failed() -> RebuiltPage {
    RebuiltPage {
        lines_as_text: 0,
        lines_total: 0,
        angle: 0.0,
        kept_reasons: BTreeMap::from([(KeepReason::PageFailed, 1)]),
        regions: vec![RebuildRegion { bounds: LayoutBox::PAGE, kind: RegionKind::Image }],
    }
}
