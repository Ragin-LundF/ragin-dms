use super::mask::Mask;

/// About 8 mm at 300 dpi: longer than the strokes of normal letters, shorter than table lines.
pub(crate) const RUN: usize = 100;
const THICK: usize = 8;
const SAMPLES: usize = 5;

/// Table rules and underlines: pixels in a horizontal or vertical run of at least [`RUN`] pixels that is
/// also thin (at most [`THICK`] pixels across), so big or bold letters are not rules.
pub(crate) fn of(mask: &Mask) -> Mask {
    let mut result = Mask::new(mask.width, mask.height);
    for y in 0..mask.height {
        for (from, to) in runs(mask.width, |x| mask.get(x, y)) {
            if is_thin(from, to, |along| thickness_at(mask.height, y, |i| mask.get(along, i))) {
                (from..to).for_each(|x| result.set(x, y, true));
            }
        }
    }
    for x in 0..mask.width {
        for (from, to) in runs(mask.height, |y| mask.get(x, y)) {
            if is_thin(from, to, |along| thickness_at(mask.width, x, |i| mask.get(i, along))) {
                (from..to).for_each(|y| result.set(x, y, true));
            }
        }
    }
    result
}

/// Every run of set pixels that is at least [`RUN`] long, as `from..to`.
fn runs(length: usize, is_on: impl Fn(usize) -> bool) -> Vec<(usize, usize)> {
    let mut found = Vec::new();
    let mut start = None;
    for i in 0..=length {
        let on = i < length && is_on(i);
        match (on, start) {
            (true, None) => start = Some(i),
            (false, Some(from)) => {
                if i - from >= RUN {
                    found.push((from, i));
                }
                start = None;
            }
            _ => {}
        }
    }
    found
}

/// Whether the median thickness across the run, sampled at five points along it, is thin.
fn is_thin(from: usize, to: usize, thickness: impl Fn(usize) -> usize) -> bool {
    let mut samples: Vec<usize> = (1..=SAMPLES).map(|i| thickness(from + (to - from) * i / (SAMPLES + 1))).collect();
    samples.sort_unstable();
    samples[SAMPLES / 2] <= THICK
}

fn thickness_at(limit: usize, at: usize, is_on: impl Fn(usize) -> bool) -> usize {
    let mut before = 0;
    let mut after = 0;
    while at > before && is_on(at - before - 1) && before <= THICK {
        before += 1;
    }
    while at + after + 1 < limit && is_on(at + after + 1) && after <= THICK {
        after += 1;
    }
    before + after + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rebuild::rect::Rect;

    #[test]
    fn a_long_thin_line_is_a_rule_and_a_thick_block_is_not() {
        let mut mask = Mask::new(300, 100);
        mask.fill(Rect::new(10, 10, 200, 2));
        mask.fill(Rect::new(10, 40, 150, 40));

        let rules = of(&mask);

        assert!(rules.get(100, 10) && rules.get(100, 11));
        assert!(!rules.get(100, 60));
    }
}
