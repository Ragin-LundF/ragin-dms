use crate::catalog::dto::owned;
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// A person to call in an emergency. `NON_EMPTY`.
#[derive(Debug, Clone, PartialEq)]
pub struct EmergencyContactDto {
    pub name: String,
    pub role: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub note: Option<String>,
}

impl EmergencyContactDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            name: fields.text("name")?.to_owned(),
            role: owned(fields.opt_text("role")?),
            phone: owned(fields.opt_text("phone")?),
            email: owned(fields.opt_text("email")?),
            note: owned(fields.opt_text("note")?),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .non_empty_text("name", Some(&self.name))
            .non_empty_text("role", self.role.as_deref())
            .non_empty_text("phone", self.phone.as_deref())
            .non_empty_text("email", self.email.as_deref())
            .non_empty_text("note", self.note.as_deref())
            .build()
    }
}
