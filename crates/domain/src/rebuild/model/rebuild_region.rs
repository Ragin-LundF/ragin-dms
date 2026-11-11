use crate::rebuild::{LayoutBox, RegionKind};

/// A region of a rebuilt page and what became of it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RebuildRegion {
    pub bounds: LayoutBox,
    pub kind: RegionKind,
}
