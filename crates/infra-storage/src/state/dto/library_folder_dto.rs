use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// One library folder in `settings.yaml`; `role` is `PRIMARY` or `SYNC`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryFolderDto {
    pub id: String,
    pub display_name: String,
    pub location: String,
    pub role: String,
}

impl LibraryFolderDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            id: fields.text("id")?.to_owned(),
            display_name: fields.text("displayName")?.to_owned(),
            location: fields.text("location")?.to_owned(),
            role: fields.text("role")?.to_owned(),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .text("id", &self.id)
            .text("displayName", &self.display_name)
            .text("location", &self.location)
            .text("role", &self.role)
            .build()
    }
}
