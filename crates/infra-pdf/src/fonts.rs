use std::sync::LazyLock;

use ttf_parser::Face;

use crate::rebuild::FontFamily;

/// Which typeface a line gets: the AI's family guess plus bold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct FontStyle {
    pub family: FontFamily,
    pub bold: bool,
}

impl FontStyle {
    pub const SANS: FontStyle = FontStyle { family: FontFamily::Sans, bold: false };
}

/// One of the bundled Liberation fonts (SIL OFL 1.1). Their metrics match Arial, Times and Courier, so a
/// rebuilt page looks the same on every system, and umlauts, ß and € work.
pub(crate) struct BundledFont {
    pub bytes: &'static [u8],
    face: Face<'static>,
    units: f32,
    cap_height: f32,
    x_height: f32,
}

const DEFAULT_CAP: f32 = 0.7;
const DEFAULT_X_HEIGHT: f32 = 0.5;
const ACCENTED_CAP: f32 = 1.22;
const LOW_MARK: f32 = 0.4;
const ACCENTED: &str = "ÄÖÜÀÁÂÉÈÊÅ";
const TALL: &str = "bdfhklitjäöüß€$%&/()[]{}|!?\"'#@";
const MIDDLE: &str = "+=<>*~^";

static FONTS: LazyLock<[BundledFont; 6]> = LazyLock::new(|| {
    [
        BundledFont::parse(include_bytes!("../fonts/LiberationSans-Regular.ttf")),
        BundledFont::parse(include_bytes!("../fonts/LiberationSans-Bold.ttf")),
        BundledFont::parse(include_bytes!("../fonts/LiberationSerif-Regular.ttf")),
        BundledFont::parse(include_bytes!("../fonts/LiberationSerif-Bold.ttf")),
        BundledFont::parse(include_bytes!("../fonts/LiberationMono-Regular.ttf")),
        BundledFont::parse(include_bytes!("../fonts/LiberationMono-Bold.ttf")),
    ]
});

pub(crate) fn font_of(style: FontStyle) -> &'static BundledFont {
    let family = match style.family {
        FontFamily::Sans => 0,
        FontFamily::Serif => 2,
        FontFamily::Mono => 4,
    };
    &FONTS[family + usize::from(style.bold)]
}

impl BundledFont {
    #[allow(clippy::expect_used)] // the fonts are compiled in; a broken one is a build defect
    fn parse(bytes: &'static [u8]) -> Self {
        let face = Face::parse(bytes, 0).expect("bundled Liberation font is valid");
        let units = f32::from(face.units_per_em());
        let share = |value: Option<i16>, default: f32| {
            value.filter(|value| *value > 0).map_or(default, |value| f32::from(value) / units)
        };
        let cap_height = share(face.capital_height(), DEFAULT_CAP);
        let x_height = share(face.x_height(), DEFAULT_X_HEIGHT);
        BundledFont { bytes, face, units, cap_height, x_height }
    }

    pub fn has_glyph(&self, character: char) -> bool {
        self.face.glyph_index(character).is_some_and(|glyph| glyph.0 != 0)
    }

    /// Whether the font has a glyph for every character of [`text`].
    pub fn can_encode(&self, text: &str) -> bool {
        text.chars().all(|character| self.has_glyph(character))
    }

    /// The width of [`text`] at size 1.
    pub fn width_at_1(&self, text: &str) -> f32 {
        text.chars()
            .filter_map(|character| self.face.glyph_index(character))
            .filter_map(|glyph| self.face.glyph_hor_advance(glyph))
            .map(|advance| f32::from(advance) / self.units)
            .sum()
    }

    /// How far the ink of [`text`] reaches above the baseline at size 1, from cap height and x-height.
    pub fn ink_top(&self, text: &str) -> f32 {
        text.chars()
            .map(|character| {
                if ACCENTED.contains(character) {
                    self.cap_height * ACCENTED_CAP
                } else if character.is_uppercase() || character.is_numeric() || TALL.contains(character) {
                    self.cap_height
                } else if character.is_alphabetic() || MIDDLE.contains(character) {
                    self.x_height
                } else {
                    self.x_height * LOW_MARK
                }
            })
            .fold(0.0, f32::max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sans_font_measures_like_arial_and_knows_umlauts_but_not_every_script() {
        let sans = font_of(FontStyle::SANS);

        assert!((sans.width_at_1("H") - 0.722).abs() < 0.001, "{}", sans.width_at_1("H"));
        assert!(sans.can_encode("Müller Straße 12 €"));
        assert!(!sans.can_encode("日本"));
        assert!(sans.ink_top("Ä") > sans.ink_top("H"));
        assert!(sans.ink_top("H") > sans.ink_top("x"));
    }
}
