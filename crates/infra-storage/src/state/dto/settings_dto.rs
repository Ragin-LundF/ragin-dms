use crate::catalog::dto::read_list;
use crate::state::dto::{AiSettingsDto, LayoutDto, LibraryFolderDto};
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// `settings.yaml`: the library folders and the preferences of this device. Every property is written, except the
/// 2.1 schedule of the archive check, which is left out while it has its default, so the file stays as 1.x wrote it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsDto {
    pub folders: Vec<LibraryFolderDto>,
    pub add_new_files_while_scanning: bool,
    pub ai: AiSettingsDto,
    pub layout: LayoutDto,
    /// `off`, `weekly` or `monthly`; `None` means weekly.
    pub health_check: Option<String>,
}

impl Default for SettingsDto {
    fn default() -> Self {
        Self {
            folders: Vec::new(),
            add_new_files_while_scanning: true,
            ai: AiSettingsDto::default(),
            layout: LayoutDto::default(),
            health_check: None,
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
            health_check: fields.opt_text("healthCheck")?.map(str::to_owned),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .node("folders", Node::Seq(self.folders.iter().map(LibraryFolderDto::write).collect()))
            .bool("addNewFilesWhileScanning", self.add_new_files_while_scanning)
            .node("ai", self.ai.write())
            .node("layout", self.layout.write())
            .opt_text("healthCheck", self.health_check.as_deref())
            .build()
    }
}
