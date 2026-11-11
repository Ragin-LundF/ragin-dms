use indexmap::IndexMap;

use crate::catalog::dto::settings::InboxDto;
use crate::catalog::dto::{extensible, owned};
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// `NON_NULL`
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutDto {
    pub inbox: Option<InboxDto>,
    pub path_template: Option<String>,
    pub default_currency: Option<String>,
    pub embed_metadata: Option<bool>,
    pub unknown: IndexMap<String, Node>,
}

extensible!(LayoutDto);

const KNOWN: [&str; 4] = ["inbox", "pathTemplate", "defaultCurrency", "embedMetadata"];

impl LayoutDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            inbox: fields.get("inbox").map(InboxDto::read).transpose()?,
            path_template: owned(fields.opt_text("pathTemplate")?),
            default_currency: owned(fields.opt_text("defaultCurrency")?),
            embed_metadata: fields.opt_bool("embedMetadata")?,
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .opt_node("inbox", self.inbox.as_ref().map(InboxDto::write))
            .opt_text("pathTemplate", self.path_template.as_deref())
            .opt_text("defaultCurrency", self.default_currency.as_deref())
            .opt_node("embedMetadata", self.embed_metadata.map(Node::Bool))
            .unknown(&self.unknown)
            .build()
    }
}
