use crate::rebuild::raster::mask::Mask;
use crate::rebuild::rect::Rect;

const FROM_SHARE: f64 = 0.4;

/// Where the letters of a line stand: in the lower 60 % of the line, the row after the sharpest drop of
/// ink per row.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_possible_wrap)] // rows of a box
pub(crate) fn of(glyph: &Mask, bounds: Rect) -> i32 {
    let (columns, rows) = glyph.clipped(bounds);
    let counts: Vec<usize> = rows.map(|y| columns.clone().filter(|&x| glyph.get(x, y)).count()).collect();
    let from = ((f64::from(bounds.height) * FROM_SHARE) as usize).max(1);
    let mut best = bounds.height;
    let mut best_drop = 0;
    for row in from..counts.len() {
        let drop = counts[row - 1].saturating_sub(counts[row]);
        if drop > best_drop {
            best_drop = drop;
            best = row as i32;
        }
    }
    bounds.y + best
}
