use crate::rebuild::{LayoutGraphic, LayoutLine};

/// What the AI found on one page: the text lines in reading order and the graphics.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PageLayout {
    pub lines: Vec<LayoutLine>,
    pub graphics: Vec<LayoutGraphic>,
}
