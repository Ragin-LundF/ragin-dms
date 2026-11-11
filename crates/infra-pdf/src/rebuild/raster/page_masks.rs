use std::sync::OnceLock;

use image::RgbImage;

use super::labels::Labels;
use super::mask::Mask;
use super::{gray, rules};

const MIN_INK: u8 = 110;
const MAX_INK: u8 = 170;
const LIGHT_OFFSET: u8 = 60;
const MAX_LIGHT: u8 = 215;
const SPECK: usize = 6;

/// The masks a page is measured on. [`ink`] is dark text, [`light`] also catches faint print and table
/// rules that scan lighter, [`rules`] are long thin runs in [`light`], and [`glyph_ink`] is ink without
/// rules, so table lines do not spoil baselines and widths.
pub(crate) struct PageMasks<'a> {
    pub page: &'a RgbImage,
    pub width: usize,
    pub height: usize,
    pub gray: Vec<u8>,
    pub ink_threshold: u8,
    pub light: Mask,
    pub rules: Mask,
    pub glyph_ink: Mask,
    near_rules: Mask,
    faint_glyph_ink: OnceLock<Mask>,
}

impl<'a> PageMasks<'a> {
    pub fn new(page: &'a RgbImage) -> Self {
        let (width, height) = (page.width() as usize, page.height() as usize);
        let gray = gray::of(page);
        let ink_threshold = gray::otsu(&gray).clamp(MIN_INK, MAX_INK);
        let light_threshold = ink_threshold.saturating_add(LIGHT_OFFSET).min(MAX_LIGHT);
        let below = |limit: u8| Mask { width, height, bits: gray.iter().map(|&value| value < limit).collect() };
        let ink = below(ink_threshold);
        let light = below(light_threshold);
        let rules = rules::of(&light);
        let near_rules = rules.dilate(1);
        let glyph_ink = Mask::without(&ink, &near_rules);
        PageMasks {
            page,
            width,
            height,
            gray,
            ink_threshold,
            light,
            rules,
            glyph_ink,
            near_rules,
            faint_glyph_ink: OnceLock::new(),
        }
    }

    /// [`light`] without rules and without specks of scanner noise, which the lighter threshold lets in.
    pub fn faint_glyph_ink(&self) -> &Mask {
        self.faint_glyph_ink.get_or_init(|| {
            let candidate = Mask::without(&self.light, &self.near_rules);
            let labels = Labels::of(&candidate);
            let bits = labels.ids.iter().map(|id| id.is_some_and(|id| labels.counts[id as usize] >= SPECK)).collect();
            Mask { width: self.width, height: self.height, bits }
        })
    }

    pub fn rgb(&self, index: usize) -> [u8; 3] {
        let x = u32::try_from(index % self.width).unwrap_or(0);
        let y = u32::try_from(index / self.width).unwrap_or(0);
        self.page.get_pixel(x, y).0
    }
}
