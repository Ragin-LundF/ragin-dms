use crate::fonts::FontStyle;
use crate::rebuild::model::KeepReason;
use crate::rebuild::rect::Rect;

pub(crate) const FULL_SCALE: f32 = 100.0;

/// A line after the checks, in pixels of the page: real text, or pixels of the scan with the reason.
#[derive(Debug, Clone)]
pub(crate) enum FittedLine {
    Kept { bounds: Rect, reason: KeepReason },
    Text(TextLine),
}

impl FittedLine {
    pub fn bounds(&self) -> Rect {
        match self {
            FittedLine::Kept { bounds, .. } => *bounds,
            FittedLine::Text(line) => line.bounds,
        }
    }
}

/// Real text in [`runs`], standing on [`baseline`], at [`size_px`] pixels in [`color`].
#[derive(Debug, Clone)]
pub(crate) struct TextLine {
    pub bounds: Rect,
    pub style: FontStyle,
    pub runs: Vec<TextRun>,
    pub baseline: i32,
    pub size_px: f32,
    pub color: [u8; 3],
}

/// One run of real text: a word, or a whole line when its words could not be matched to the ink.
#[derive(Debug, Clone)]
pub(crate) struct TextRun {
    pub text: String,
    pub ink: Rect,
    pub scale_percent: f32,
}
