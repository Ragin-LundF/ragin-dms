use crate::rebuild::{FontFamily, KeepReason, LayoutBox};

/// One printed text line as the AI read it. [`Self::legible`] is the AI's own certainty about every character.
/// [`Self::kept`] is set when the line must stay an image for a reason found before fitting it to the ink.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutLine {
    pub bounds: LayoutBox,
    pub text: String,
    pub bold: bool,
    pub family: FontFamily,
    pub legible: bool,
    pub kept: Option<KeepReason>,
}
