use crate::rebuild::raster::mask::Mask;
use crate::rebuild::raster::page_masks::PageMasks;
use crate::rebuild::rect::Rect;

/// The colour a line is printed in: the mean colour of the darker half of the ink in [`bounds`], since
/// antialiased edges are lighter than the stroke. Faint print has no pixel that dark, so then the darker
/// half of its [`glyph`] pixels counts.
pub(crate) fn of(masks: &PageMasks, glyph: &Mask, bounds: Rect) -> [u8; 3] {
    let pixels = pixels_of(masks.width, glyph, bounds);
    let mut chosen: Vec<usize> = pixels.iter().copied().filter(|&i| masks.gray[i] < masks.ink_threshold / 2).collect();
    if chosen.is_empty() {
        chosen = pixels.into_iter().filter(|&i| glyph.bits[i]).collect();
        chosen.sort_by_key(|&i| masks.gray[i]);
        chosen.truncate((chosen.len() / 2).max(1));
    }
    if chosen.is_empty() {
        return [0, 0, 0];
    }
    let mut sums = [0_usize; 3];
    for &i in &chosen {
        let rgb = masks.rgb(i);
        for channel in 0..3 {
            sums[channel] += usize::from(rgb[channel]);
        }
    }
    sums.map(|sum| u8::try_from(sum / chosen.len()).unwrap_or(u8::MAX))
}

fn pixels_of(width: usize, glyph: &Mask, bounds: Rect) -> Vec<usize> {
    let (columns, rows) = glyph.clipped(bounds);
    rows.flat_map(|y| columns.clone().map(move |x| y * width + x)).collect()
}
