use std::ops::Range;

use crate::rebuild::raster::mask::Mask;
use crate::rebuild::rect::Rect;

const X_HEIGHT_TO_EM: f32 = 0.7;
const MIN_GAP_EM: f32 = 0.15;
const GAP_LEAD: f32 = 1.2;

/// Ink boxes for [`count`] words, separated by the count-1 widest gaps between ink columns above the
/// [`baseline`], if they are clearly wider than the gaps inside words; `None` if the words cannot be
/// matched.
#[allow(clippy::cast_precision_loss)] // gap widths of a page
pub(crate) fn of(glyph: &Mask, bounds: Rect, baseline: i32, count: usize) -> Option<Vec<Rect>> {
    let area = tighten(glyph, bounds, baseline);
    if count == 1 {
        return Some(vec![area]);
    }
    let mut gaps = gaps(glyph, area, bounds.y, baseline);
    if gaps.len() < count - 1 {
        return None;
    }
    // Stable, as Kotlin's sortedByDescending: equal gaps keep their order from left to right.
    gaps.sort_by_key(|gap| std::cmp::Reverse(gap.len()));
    let narrowest_chosen = gaps[count - 2].len() as f32;
    let widest_other = gaps.get(count - 1).map_or(0.0, |gap| gap.len() as f32);
    let em = (baseline - bounds.y) as f32 / X_HEIGHT_TO_EM;
    if narrowest_chosen < em * MIN_GAP_EM || narrowest_chosen < widest_other * GAP_LEAD {
        return None;
    }
    let mut cuts: Vec<Range<i32>> = gaps[..count - 1].to_vec();
    cuts.sort_by_key(|gap| gap.start);
    let mut left = area.x;
    let rights = cuts.iter().map(|cut| cut.start).chain([area.right()]);
    Some(
        rights
            .enumerate()
            .map(|(i, right)| {
                let word = tighten(glyph, Rect::new(left, bounds.y, right - left, bounds.height), baseline);
                left = cuts.get(i).map_or(right, |cut| cut.end);
                word
            })
            .collect(),
    )
}

/// [`bounds`] narrowed to the ink columns above the [`baseline`].
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap, clippy::cast_sign_loss)] // page coordinates
pub(crate) fn tighten(glyph: &Mask, bounds: Rect, baseline: i32) -> Rect {
    let rows = bounds.y..baseline.max(bounds.y + 1);
    let (columns, _) = glyph.clipped(bounds);
    let inked: Vec<usize> = columns.filter(|&x| rows.clone().any(|y| glyph.get(x, y as usize))).collect();
    match (inked.first(), inked.last()) {
        (Some(&first), Some(&last)) => Rect::new(first as i32, bounds.y, (last - first + 1) as i32, bounds.height),
        _ => bounds,
    }
}

/// The runs of columns of [`area`] without ink between [`top`] and [`baseline`]; a run that reaches the
/// right edge is no gap.
#[allow(clippy::cast_sign_loss)] // page coordinates
fn gaps(glyph: &Mask, area: Rect, top: i32, baseline: i32) -> Vec<Range<i32>> {
    let mut gaps = Vec::new();
    let mut start: Option<i32> = None;
    for x in area.x..area.right() {
        let on = (top..baseline).any(|y| glyph.get(x as usize, y as usize));
        match (on, start) {
            (false, None) => start = Some(x),
            (true, Some(from)) => {
                gaps.push(from..x);
                start = None;
            }
            _ => {}
        }
    }
    gaps
}
