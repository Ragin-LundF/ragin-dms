use crate::catalog::dto::read_list;
use crate::state::dto::{AiSettingsDto, LayoutDto, LibraryFolderDto};
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// `settings.yaml`: the library folders and the preferences of this device. Every property is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsDto {
    pub folders: Vec<LibraryFolderDto>,
    pub add_new_files_while_scanning: bool,
    pub ai: AiSettingsDto,
    pub layout: LayoutDto,
}

impl Default for SettingsDto {
    fn default() -> Self {
        Self {
            folders: Vec::new(),
            add_new_files_while_scanning: true,
            ai: AiSettingsDto::default(),
            layout: LayoutDto::default(),
        }
    }
}

impl SettingsDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            folders: read_list(fields, "folders", LibraryFolderDto::read)?,
            add_new_files_while_scanning: fields.bool_or("addNewFilesWhileScanning", true)?,
            ai: fields.get("ai").map(AiSettingsDto::read).transpose()?.unwrap_or_default(),
            layout: fields.get("layout").map(LayoutDto::read).transpose()?.unwrap_or_default(),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .node("folders", Node::Seq(self.folders.iter().map(LibraryFolderDto::write).collect()))
            .bool("addNewFilesWhileScanning", self.add_new_files_while_scanning)
            .node("ai", self.ai.write())
            .node("layout", self.layout.write())
            .build()
    }
}
