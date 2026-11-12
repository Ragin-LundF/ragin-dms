use indexmap::IndexMap;

use crate::catalog::dto::settings::FieldConditionDto;
use crate::catalog::dto::{extensible, owned, owned_texts, read_list, texts_node};
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// A document query; only the conditions that are set are written. Dates are ISO-8601. `NON_EMPTY`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct QueryDto {
    pub text: Option<String>,
    pub category: Option<String>,
    pub uncategorized: Option<bool>,
    pub tags_all: Vec<String>,
    pub tags_any: Vec<String>,
    pub tags_none: Vec<String>,
    pub untagged: Option<bool>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub correspondent: Option<String>,
    pub folder: Option<String>,
    pub fields: Vec<FieldConditionDto>,
    pub added_since: Option<String>,
    pub content_text: Option<String>,
    pub tax_year: Option<i32>,
    pub tax_area: Option<String>,
    pub tax_tag_fallback: Option<String>,
    pub unknown: IndexMap<String, Node>,
}

extensible!(QueryDto);

const KNOWN: [&str; 17] = [
    "text",
    "category",
    "uncategorized",
    "tagsAll",
    "tagsAny",
    "tagsNone",
    "untagged",
    "from",
    "to",
    "correspondent",
    "folder",
    "fields",
    "addedSince",
    "contentText",
    "taxYear",
    "taxArea",
    "taxTagFallback",
];

impl QueryDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            text: owned(fields.opt_text("text")?),
            category: owned(fields.opt_text("category")?),
            uncategorized: fields.opt_bool("uncategorized")?,
            tags_all: owned_texts(fields, "tagsAll")?,
            tags_any: owned_texts(fields, "tagsAny")?,
            tags_none: owned_texts(fields, "tagsNone")?,
            untagged: fields.opt_bool("untagged")?,
            from: owned(fields.opt_text("from")?),
            to: owned(fields.opt_text("to")?),
            correspondent: owned(fields.opt_text("correspondent")?),
            folder: owned(fields.opt_text("folder")?),
            fields: read_list(fields, "fields", FieldConditionDto::read)?,
            added_since: owned(fields.opt_text("addedSince")?),
            content_text: owned(fields.opt_text("contentText")?),
            tax_year: fields.opt_i32("taxYear")?,
            tax_area: owned(fields.opt_text("taxArea")?),
            tax_tag_fallback: owned(fields.opt_text("taxTagFallback")?),
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .non_empty_text("text", self.text.as_deref())
            .non_empty_text("category", self.category.as_deref())
            .opt_node("uncategorized", self.uncategorized.map(Node::Bool))
            .non_empty_list("tagsAll", texts_node(&self.tags_all))
            .non_empty_list("tagsAny", texts_node(&self.tags_any))
            .non_empty_list("tagsNone", texts_node(&self.tags_none))
            .opt_node("untagged", self.untagged.map(Node::Bool))
            .non_empty_text("from", self.from.as_deref())
            .non_empty_text("to", self.to.as_deref())
            .non_empty_text("correspondent", self.correspondent.as_deref())
            .non_empty_text("folder", self.folder.as_deref())
            .non_empty_list("fields", self.fields.iter().map(FieldConditionDto::write).collect())
            .non_empty_text("addedSince", self.added_since.as_deref())
            .non_empty_text("contentText", self.content_text.as_deref())
            .opt_node("taxYear", self.tax_year.map(Node::int))
            .non_empty_text("taxArea", self.tax_area.as_deref())
            .non_empty_text("taxTagFallback", self.tax_tag_fallback.as_deref())
            .unknown(&self.unknown)
            .build()
    }
}
