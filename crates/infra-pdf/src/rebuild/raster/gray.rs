use image::RgbImage;

const LEVELS: usize = 256;

/// The grey value 0..255 of a pixel (ITU-R 601 weights, as the Kotlin app).
pub(crate) fn value_of(rgb: [u8; 3]) -> u8 {
    let [red, green, blue] = rgb.map(u32::from);
    u8::try_from((red * 299 + green * 587 + blue * 114) / 1000).unwrap_or(u8::MAX)
}

/// The grey value of every pixel of [`image`], row by row.
pub(crate) fn of(image: &RgbImage) -> Vec<u8> {
    image.pixels().map(|pixel| value_of(pixel.0)).collect()
}

/// Otsu's threshold: the grey value that best splits [`gray`] into two classes.
#[allow(clippy::cast_precision_loss)] // pixel counts stay far below 2^52
pub(crate) fn otsu(gray: &[u8]) -> u8 {
    let mut histogram = [0_u64; LEVELS];
    for &value in gray {
        histogram[usize::from(value)] += 1;
    }
    let total = gray.len() as f64;
    let sum: f64 = histogram.iter().enumerate().map(|(level, &count)| level as f64 * count as f64).sum();
    let mut sum_background = 0.0;
    let mut weight_background = 0.0;
    let mut best = 0.0;
    let mut threshold = LEVELS / 2;
    for (level, &count) in histogram.iter().enumerate() {
        weight_background += count as f64;
        let weight_foreground = total - weight_background;
        sum_background += level as f64 * count as f64;
        if weight_background == 0.0 || weight_foreground == 0.0 {
            continue;
        }
        let difference = sum_background / weight_background - (sum - sum_background) / weight_foreground;
        let between = weight_background * weight_foreground * difference * difference;
        if between > best {
            best = between;
            threshold = level;
        }
    }
    u8::try_from(threshold).unwrap_or(u8::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn otsu_splits_ink_from_paper() {
        let mut gray = vec![250_u8; 900];
        gray.extend(std::iter::repeat_n(20_u8, 100));

        let threshold = otsu(&gray);

        assert!((20..250).contains(&threshold), "{threshold}");
    }
}
