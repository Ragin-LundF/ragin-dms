use dms_application::port::state::LayoutPreferences;

use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// The panel layout part of `settings.yaml`. Every field is optional, so older files keep loading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutDto {
    pub sidebar_width: i32,
    pub sidebar_folded: bool,
    pub folder_tree_width: i32,
    pub folder_tree_hidden: bool,
}

impl Default for LayoutDto {
    fn default() -> Self {
        Self {
            sidebar_width: LayoutPreferences::DEFAULT_SIDEBAR_WIDTH,
            sidebar_folded: false,
            folder_tree_width: LayoutPreferences::DEFAULT_FOLDER_TREE_WIDTH,
            folder_tree_hidden: false,
        }
    }
}

impl LayoutDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        let default = Self::default();
        Ok(Self {
            sidebar_width: fields.opt_i32("sidebarWidth")?.unwrap_or(default.sidebar_width),
            sidebar_folded: fields.bool_or("sidebarFolded", default.sidebar_folded)?,
            folder_tree_width: fields.opt_i32("folderTreeWidth")?.unwrap_or(default.folder_tree_width),
            folder_tree_hidden: fields.bool_or("folderTreeHidden", default.folder_tree_hidden)?,
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .int("sidebarWidth", self.sidebar_width)
            .bool("sidebarFolded", self.sidebar_folded)
            .int("folderTreeWidth", self.folder_tree_width)
            .bool("folderTreeHidden", self.folder_tree_hidden)
            .build()
    }
}
