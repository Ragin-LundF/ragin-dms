use indexmap::IndexMap;

use crate::catalog::dto::{extensible, owned, owned_texts, texts_node};
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// The owner's details for letters. `NON_EMPTY`.
#[derive(Debug, Clone, PartialEq)]
pub struct OwnerDto {
    pub name: String,
    pub address_lines: Vec<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub unknown: IndexMap<String, Node>,
}

extensible!(OwnerDto);

const KNOWN: [&str; 4] = ["name", "addressLines", "email", "phone"];

impl OwnerDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            name: fields.text("name")?.to_owned(),
            address_lines: owned_texts(fields, "addressLines")?,
            email: owned(fields.opt_text("email")?),
            phone: owned(fields.opt_text("phone")?),
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .non_empty_text("name", Some(&self.name))
            .non_empty_list("addressLines", texts_node(&self.address_lines))
            .non_empty_text("email", self.email.as_deref())
            .non_empty_text("phone", self.phone.as_deref())
            .unknown(&self.unknown)
            .build()
    }
}
