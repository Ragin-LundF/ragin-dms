//! What the PDF tools exchange with Rust.

use serde::{Deserialize, Serialize};
use specta::Type;

/// Which PDF tools quick look offers for a document. `pdf` false hides the section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PdfToolsDto {
    pub pdf: bool,
    /// The file name, as the page editor's title shows it.
    pub name: String,
    /// Whether ocrmypdf is installed, for "Precise OCR".
    pub ocr_available: bool,
    /// Whether "Convert to real PDF" can take this document (a PDF without text layer).
    pub rebuildable: bool,
}

/// A PDF to merge: its title or file name, and its path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MergeCandidateDto {
    pub document_id: String,
    pub name: String,
    pub path: String,
}

/// "Merge with…" opened on a document: it comes first, and the new file's suggested name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MergeStartDto {
    pub first: MergeCandidateDto,
    pub name: String,
}

/// One page in the page editor: page `source` of the file (from 0), turned by `rotation` degrees, `deleted` to
/// leave it out, `split_after` to start a new document after it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PageItemDto {
    pub source: u32,
    pub rotation: i32,
    pub deleted: bool,
    pub split_after: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum RebuildChoiceDto {
    KeepOriginal,
    ReplaceWithRebuild,
    KeepBoth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum RegionKindDto {
    Text,
    KeptLine,
    Image,
}

/// What became of one area of a rebuilt page, in page coordinates from 0 to 1000 (rounded; a unit is far below a
/// pixel of the comparison).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RegionDto {
    pub kind: RegionKindDto,
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

/// One rebuilt page: how many lines became real text, why lines stayed images ("Kept as images: …") and the areas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RebuiltPageDto {
    pub lines_as_text: u32,
    pub lines_total: u32,
    pub kept: Option<String>,
    pub regions: Vec<RegionDto>,
}

/// The open comparison of a scan with its rebuild; it stays until the user chooses what to keep. Sizes in bytes;
/// `rebuild_id` names the rebuilt file's page pictures.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RebuildCompareDto {
    pub document_id: String,
    pub rebuild_id: String,
    pub name: String,
    pub path: String,
    pub original_size: f64,
    pub rebuilt_size: f64,
    pub pages: Vec<RebuiltPageDto>,
}
