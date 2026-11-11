use std::ops::RangeInclusive;

/// How the library's panels sit on this device: the width of the library sidebar and of the folder tree in
/// density-independent pixels, and whether each is folded away.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayoutPreferences {
    pub sidebar_width: i32,
    pub sidebar_folded: bool,
    pub folder_tree_width: i32,
    pub folder_tree_hidden: bool,
}

impl LayoutPreferences {
    pub const DEFAULT_SIDEBAR_WIDTH: i32 = 200;
    pub const DEFAULT_FOLDER_TREE_WIDTH: i32 = 240;
    pub const SIDEBAR_WIDTHS: RangeInclusive<i32> = 150..=360;
    pub const FOLDER_TREE_WIDTHS: RangeInclusive<i32> = 160..=480;
}

impl Default for LayoutPreferences {
    fn default() -> Self {
        Self {
            sidebar_width: Self::DEFAULT_SIDEBAR_WIDTH,
            sidebar_folded: false,
            folder_tree_width: Self::DEFAULT_FOLDER_TREE_WIDTH,
            folder_tree_hidden: false,
        }
    }
}
