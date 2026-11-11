use crate::rebuild::rect::Rect;

/// A boolean image the size of the page, row by row in [`bits`].
#[derive(Clone)]
pub(crate) struct Mask {
    pub width: usize,
    pub height: usize,
    pub bits: Vec<bool>,
}

impl Mask {
    pub fn new(width: usize, height: usize) -> Self {
        Mask { width, height, bits: vec![false; width * height] }
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        self.bits[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        self.bits[y * self.width + x] = value;
    }

    /// Square dilation by [`radius`], as two running-count passes, so the cost does not grow with it.
    pub fn dilate(&self, radius: usize) -> Mask {
        let (width, height) = (self.width, self.height);
        let mut horizontal = vec![false; self.bits.len()];
        for y in 0..height {
            slide(width, radius, |x| self.bits[y * width + x], |x, on| horizontal[y * width + x] = on);
        }
        let mut result = vec![false; self.bits.len()];
        for x in 0..width {
            slide(height, radius, |y| horizontal[y * width + x], |y, on| result[y * width + x] = on);
        }
        Mask { width, height, bits: result }
    }

    pub fn fill(&mut self, rect: Rect) {
        let (xs, ys) = self.clipped(rect);
        for y in ys {
            self.bits[y * self.width + xs.start..y * self.width + xs.end].fill(true);
        }
    }

    /// The columns and rows of [`rect`] that lie inside the mask.
    pub fn clipped(&self, rect: Rect) -> (std::ops::Range<usize>, std::ops::Range<usize>) {
        let clamp = |value: i32, limit: usize| usize::try_from(value.max(0)).unwrap_or(0).min(limit);
        let xs = clamp(rect.x, self.width)..clamp(rect.right(), self.width);
        let ys = clamp(rect.y, self.height)..clamp(rect.bottom(), self.height);
        (xs.start..xs.end.max(xs.start), ys.start..ys.end.max(ys.start))
    }

    /// The pixels set in [`first`] and not in [`second`].
    pub fn without(first: &Mask, second: &Mask) -> Mask {
        let bits = first.bits.iter().zip(&second.bits).map(|(&a, &b)| a && !b).collect();
        Mask { width: first.width, height: first.height, bits }
    }
}

/// A running count over one row or column: [`emit`] gets whether a set pixel lies within [`radius`].
fn slide(length: usize, radius: usize, is_on: impl Fn(usize) -> bool, mut emit: impl FnMut(usize, bool)) {
    let mut count = (0..radius.min(length)).filter(|&i| is_on(i)).count();
    for i in 0..length {
        if i + radius < length && is_on(i + radius) {
            count += 1;
        }
        if i > radius && is_on(i - radius - 1) {
            count -= 1;
        }
        emit(i, count > 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dilation_grows_a_pixel_into_a_square() {
        let mut mask = Mask::new(7, 7);
        mask.set(3, 3, true);

        let grown = mask.dilate(2);

        assert_eq!(grown.bits.iter().filter(|&&on| on).count(), 25);
        assert!(grown.get(1, 1) && grown.get(5, 5) && !grown.get(0, 3));
    }
}
