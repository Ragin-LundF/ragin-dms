use serde::Deserialize;

use crate::classify::suggestion_dto::null_as_default;

/// The AI's answer for one page, as the layout schema asks for it. `null` stands for the default, as with Kotlin's
/// `coerceInputValues`.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct LayoutAnswerDto {
    #[serde(default, deserialize_with = "null_as_default")]
    pub lines: Vec<LayoutLineDto>,
    #[serde(default, deserialize_with = "null_as_default")]
    pub graphics: Vec<LayoutGraphicDto>,
}

/// One text line: `bbox` is x0, y0, x1, y1, in 0..1000 or in pixels of the sent image.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct LayoutLineDto {
    #[serde(default, deserialize_with = "null_as_default")]
    pub bbox: Vec<f64>,
    #[serde(default, deserialize_with = "null_as_default")]
    pub text: String,
    #[serde(default, deserialize_with = "null_as_default")]
    pub bold: bool,
    #[serde(default)]
    pub font: Option<String>,
    #[serde(default, deserialize_with = "null_as_default")]
    pub legible: bool,
}

/// An area that is no printed text, such as a logo or a stamp.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct LayoutGraphicDto {
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default, deserialize_with = "null_as_default")]
    pub bbox: Vec<f64>,
}
