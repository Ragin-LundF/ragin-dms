//! The values of a rebuilt scan: what the AI read on a page and what became of it.

mod font_family;
mod keep_reason;
pub mod model;
mod region_kind;

pub use font_family::FontFamily;
pub use keep_reason::KeepReason;
pub use model::{LayoutBox, LayoutGraphic, LayoutLine, PageLayout, RebuildRegion, RebuiltPage};
pub use region_kind::RegionKind;
