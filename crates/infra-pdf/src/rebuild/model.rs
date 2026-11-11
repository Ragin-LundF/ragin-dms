use std::collections::BTreeMap;

use crate::images::PageImage;

/// A box on a page in units of 0..1000 of the page width and height, origin top left. No pixel sizes
/// cross the ports; each side converts to its own image.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutBox {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

impl LayoutBox {
    pub const SCALE: f64 = 1000.0;
    pub const PAGE: LayoutBox = LayoutBox { left: 0.0, top: 0.0, right: Self::SCALE, bottom: Self::SCALE };
}

/// The kind of typeface a text line is printed in, as the AI saw it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontFamily {
    Sans,
    Serif,
    Mono,
}

/// Why a text line of a rebuilt page stayed pixels of the scan instead of becoming real text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeepReason {
    /// The AI was not sure about every character.
    NotSure,
    /// No ink was found where the AI placed the line.
    NoInk,
    /// A stamp, signature or handwriting covers the line.
    Covered,
    /// The font has no glyph for a character of the line.
    NotEncodable,
    /// The text is wider or narrower than the ink: words are missing or invented.
    TextMismatch,
    /// The AI lost characters, and reading the line again did not recover them (set by the caller).
    LostCharacters,
    /// The AI could not read the page, so the whole page stayed as it was.
    PageFailed,
}

/// What a region of a rebuilt page is, for the comparison overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionKind {
    Text,
    KeptLine,
    Image,
}

/// One printed text line as the AI read it. [`legible`] is the AI's own certainty about every character;
/// [`kept`] is set when the line must stay an image for a reason found before fitting it to the ink.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutLine {
    pub bounds: LayoutBox,
    pub text: String,
    pub bold: bool,
    pub family: FontFamily,
    pub legible: bool,
    pub kept: Option<KeepReason>,
}

/// An area that is no printed text: a logo, stamp, signature, photo or code, with the AI's [`kind`].
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutGraphic {
    pub kind: String,
    pub bounds: LayoutBox,
}

/// What the AI found on one page: the text lines in reading order and the graphics.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PageLayout {
    pub lines: Vec<LayoutLine>,
    pub graphics: Vec<LayoutGraphic>,
}

/// A region of a rebuilt page and what became of it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RebuildRegion {
    pub bounds: LayoutBox,
    pub kind: RegionKind,
}

/// The result for one page: [`lines_as_text`] of [`lines_total`] lines became real text, the page was
/// straightened by [`angle`] degrees, and [`kept_reasons`] counts why the other lines stayed images.
#[derive(Debug, Clone, PartialEq)]
pub struct RebuiltPage {
    pub lines_as_text: usize,
    pub lines_total: usize,
    pub angle: f64,
    pub kept_reasons: BTreeMap<KeepReason, usize>,
    pub regions: Vec<RebuildRegion>,
}

/// Page [`page_index`] of a scan, rendered at 300 dpi and straightened by [`angle`] degrees (clockwise
/// positive), as PNG.
#[derive(Debug, Clone, PartialEq)]
pub struct PreparedPage {
    pub page_index: usize,
    pub image: PageImage,
    pub angle: f64,
}

/// What the rebuild writes for one page.
#[derive(Debug, Clone, PartialEq)]
pub enum PageInput {
    /// A page the AI read: rebuilt from [`layout`] on the straightened [`prepared`] page.
    Read { prepared: PreparedPage, layout: PageLayout },
    /// A page the AI could not read: page [`page_index`] of the original stays as it is.
    Failed { page_index: usize },
}

/// A rebuilt PDF waiting outside the library for the user's choice. [`id`] names it for the rebuilder
/// only; [`size`] is its size in bytes; [`source_hash`] is the content of the scan it was made from.
#[derive(Debug, Clone, PartialEq)]
pub struct RebuiltFile {
    pub id: String,
    pub pages: Vec<RebuiltPage>,
    pub size: u64,
    pub source_hash: String,
}
