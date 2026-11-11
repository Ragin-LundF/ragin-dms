use image::{Rgb, RgbImage};

use super::gray;

const STEP: u32 = 4;
const MAX_ANGLE: f64 = 15.0;
const MIN_ANGLE: f64 = 0.15;
const COARSE: f64 = 0.5;
const FINE: f64 = 0.05;
const MIN_INK_PIXELS: usize = 200;
const MIN_INK: u8 = 110;
const MAX_INK: u8 = 170;
const MARGIN_X: u32 = 20;
const MARGIN_Y: u32 = 30;
const WHITE: f64 = 255.0;

/// The skew of [`page`] in degrees, clockwise positive, or 0 when it is too small to matter. Text lines
/// only give sharp peaks in the ink-per-row profile when they run horizontally, so the angle with the
/// sharpest profile is the skew of the page.
pub(crate) fn angle_of(page: &RgbImage) -> f64 {
    let angle = measure(page);
    if angle.abs() < MIN_ANGLE { 0.0 } else { angle }
}

/// Coarse search in 0.5° steps, then 0.05° steps around the best one, on a page reduced to a quarter.
fn measure(page: &RgbImage) -> f64 {
    let Some(mut profile) = InkProfile::of(page) else {
        return 0.0;
    };
    let coarse = profile.best(&steps(-MAX_ANGLE, MAX_ANGLE, COARSE));
    profile.best(&steps(coarse - COARSE, coarse + COARSE, FINE))
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)] // at most 61 steps
fn steps(from: f64, to: f64, step: f64) -> Vec<f64> {
    let count = ((to - from) / step + 0.5).floor() as usize + 1;
    (0..count).map(|i| from + step * i as f64).collect()
}

/// The ink pixels of a reduced page, away from the margins, which often hold scanner shadows and holes.
struct InkProfile {
    xs: Vec<i64>,
    ys: Vec<i64>,
    width: i64,
    height: i64,
    rows: Vec<u64>,
}

impl InkProfile {
    fn of(page: &RgbImage) -> Option<InkProfile> {
        let (width, height) = (page.width() / STEP, page.height() / STEP);
        let gray: Vec<u8> = (0..width * height)
            .map(|i| gray::value_of(page.get_pixel((i % width) * STEP, (i / width) * STEP).0))
            .collect();
        let limit = gray::otsu(&gray).clamp(MIN_INK, MAX_INK);
        let (mut xs, mut ys) = (Vec::new(), Vec::new());
        for y in height / MARGIN_Y..height - height / MARGIN_Y {
            for x in width / MARGIN_X..width - width / MARGIN_X {
                if gray[(y * width + x) as usize] < limit {
                    xs.push(i64::from(x));
                    ys.push(i64::from(y));
                }
            }
        }
        if xs.len() < MIN_INK_PIXELS {
            return None;
        }
        let (width, height) = (i64::from(width), i64::from(height));
        Some(InkProfile { xs, ys, width, height, rows: vec![0; usize::try_from(height * 2).unwrap_or(0)] })
    }

    /// The first of [`angles`] with the highest score, as Kotlin's `maxBy`.
    fn best(&mut self, angles: &[f64]) -> f64 {
        let mut best = (f64::MIN, 0.0);
        for &angle in angles {
            let score = self.score(angle);
            if score > best.0 {
                best = (score, angle);
            }
        }
        best.1
    }

    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)] // reduced page coordinates
    fn score(&mut self, degrees: f64) -> f64 {
        self.rows.fill(0);
        let slope = degrees.to_radians().tan();
        for (&x, &y) in self.xs.iter().zip(&self.ys) {
            let row = (y as f64 - (x - self.width / 2) as f64 * slope + 0.5).floor() as i64 + self.height / 2;
            if let Some(count) = usize::try_from(row).ok().and_then(|row| self.rows.get_mut(row)) {
                *count += 1;
            }
        }
        self.rows.iter().map(|&count| (count * count) as f64).sum()
    }
}

/// [`page`] turned by [`degrees`] clockwise about its centre, bicubic, on white.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_possible_wrap)] // pixel coordinates
pub(crate) fn rotate(page: &RgbImage, degrees: f64) -> RgbImage {
    let (width, height) = (page.width(), page.height());
    let (centre_x, centre_y) = (f64::from(width) / 2.0, f64::from(height) / 2.0);
    let (sin, cos) = degrees.to_radians().sin_cos();
    RgbImage::from_fn(width, height, |x, y| {
        // Inverse mapping from the centre of the target pixel into the source.
        let dx = f64::from(x) + 0.5 - centre_x;
        let dy = f64::from(y) + 0.5 - centre_y;
        let source_x = cos * dx + sin * dy + centre_x - 0.5;
        let source_y = -sin * dx + cos * dy + centre_y - 0.5;
        if source_x < -0.5 || source_y < -0.5 || source_x > f64::from(width) - 0.5 || source_y > f64::from(height) - 0.5
        {
            return Rgb([255, 255, 255]);
        }
        bicubic(page, source_x, source_y)
    })
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_possible_wrap)] // clamped pixel values
fn bicubic(page: &RgbImage, x: f64, y: f64) -> Rgb<u8> {
    let (base_x, base_y) = (x.floor(), y.floor());
    let (fraction_x, fraction_y) = (x - base_x, y - base_y);
    let weights_x = cubic_weights(fraction_x);
    let weights_y = cubic_weights(fraction_y);
    let max_x = i64::from(page.width()) - 1;
    let max_y = i64::from(page.height()) - 1;
    let mut sum = [0.0_f64; 3];
    for (j, weight_y) in weights_y.iter().enumerate() {
        let source_y = (base_y as i64 - 1 + j as i64).clamp(0, max_y) as u32;
        for (i, weight_x) in weights_x.iter().enumerate() {
            let source_x = (base_x as i64 - 1 + i as i64).clamp(0, max_x) as u32;
            let pixel = page.get_pixel(source_x, source_y).0;
            for channel in 0..3 {
                sum[channel] += f64::from(pixel[channel]) * weight_x * weight_y;
            }
        }
    }
    Rgb(sum.map(|value| value.round().clamp(0.0, WHITE) as u8))
}

/// Catmull-Rom weights (a = -0.5, as Java2D's bicubic) for the four neighbours at -1, 0, 1, 2.
fn cubic_weights(t: f64) -> [f64; 4] {
    let a = -0.5;
    let far = |d: f64| a * d * d * d - 5.0 * a * d * d + 8.0 * a * d - 4.0 * a;
    let near = |d: f64| (a + 2.0) * d * d * d - (a + 3.0) * d * d + 1.0;
    [far(1.0 + t), near(t), near(1.0 - t), far(2.0 - t)]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Thin black bars like text lines on white.
    fn striped(width: u32, height: u32) -> RgbImage {
        RgbImage::from_fn(width, height, |x, y| {
            let in_line = y % 50 < 8 && (200..width - 200).contains(&x);
            if in_line { Rgb([0, 0, 0]) } else { Rgb([255, 255, 255]) }
        })
    }

    #[test]
    fn a_page_turned_clockwise_measures_a_positive_angle_and_turns_back_straight() {
        let page = striped(2400, 1200);
        let turned = rotate(&page, 2.0);

        let angle = angle_of(&turned);
        let straight = angle_of(&rotate(&turned, -angle));

        assert!((angle - 2.0).abs() <= 0.1, "{angle}");
        assert!(straight.abs() < 0.001, "{straight}");
        assert_eq!(angle_of(&page), 0.0);
    }

    #[test]
    fn an_empty_page_has_no_skew() {
        assert_eq!(angle_of(&RgbImage::from_pixel(400, 400, Rgb([255, 255, 255]))), 0.0);
    }
}
