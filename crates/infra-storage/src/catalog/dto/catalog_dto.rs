use indexmap::IndexMap;

use crate::catalog::dto::rule::RuleDto;
use crate::catalog::dto::settings::{CorrespondentDto, FieldDefinitionDto, LayoutDto, SavedSearchDto};
use crate::catalog::dto::{CategoryDto, DocumentDto, LinkDto};
use crate::yaml::{Fields, MapWriter, Node, YamlError};

use super::{extensible, owned_texts, read_list, texts_node};

/// The catalog file `dms-catalog.yaml`. Everything after `documents` was added in 1.1 as optional fields, so
/// older versions keep it as unknown fields and older files still load. `NON_EMPTY`.
#[derive(Debug, Clone, PartialEq)]
pub struct CatalogDto {
    pub format_version: i32,
    pub library_id: String,
    pub revision: i64,
    pub categories: Vec<CategoryDto>,
    pub tags: Vec<String>,
    pub documents: Vec<DocumentDto>,
    pub links: Vec<LinkDto>,
    pub field_definitions: Vec<FieldDefinitionDto>,
    pub saved_searches: Vec<SavedSearchDto>,
    pub rules: Vec<RuleDto>,
    pub correspondents: Vec<CorrespondentDto>,
    pub layout: Option<LayoutDto>,
    pub unknown: IndexMap<String, Node>,
}

extensible!(CatalogDto);

const KNOWN: [&str; 12] = [
    "formatVersion",
    "libraryId",
    "revision",
    "layout",
    "fieldDefinitions",
    "savedSearches",
    "correspondents",
    "rules",
    "categories",
    "tags",
    "documents",
    "links",
];

impl CatalogDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        let format_version = fields.opt_i32("formatVersion")?.ok_or_else(|| missing("formatVersion"))?;
        Ok(Self {
            format_version,
            library_id: fields.text("libraryId")?.to_owned(),
            revision: fields.i64("revision")?,
            categories: read_list(fields, "categories", CategoryDto::read)?,
            tags: owned_texts(fields, "tags")?,
            documents: read_list(fields, "documents", DocumentDto::read)?,
            links: read_list(fields, "links", LinkDto::read)?,
            field_definitions: read_list(fields, "fieldDefinitions", FieldDefinitionDto::read)?,
            saved_searches: read_list(fields, "savedSearches", SavedSearchDto::read)?,
            rules: read_list(fields, "rules", RuleDto::read)?,
            correspondents: read_list(fields, "correspondents", CorrespondentDto::read)?,
            layout: fields.get("layout").map(LayoutDto::read).transpose()?,
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .int("formatVersion", self.format_version)
            .non_empty_text("libraryId", Some(&self.library_id))
            .int("revision", self.revision)
            .opt_node("layout", self.layout.as_ref().map(LayoutDto::write))
            .non_empty_list("fieldDefinitions", self.field_definitions.iter().map(FieldDefinitionDto::write).collect())
            .non_empty_list("savedSearches", self.saved_searches.iter().map(SavedSearchDto::write).collect())
            .non_empty_list("correspondents", self.correspondents.iter().map(CorrespondentDto::write).collect())
            .non_empty_list("rules", self.rules.iter().map(RuleDto::write).collect())
            .non_empty_list("categories", self.categories.iter().map(CategoryDto::write).collect())
            .non_empty_list("tags", texts_node(&self.tags))
            .non_empty_list("documents", self.documents.iter().map(DocumentDto::write).collect())
            .non_empty_list("links", self.links.iter().map(LinkDto::write).collect())
            .unknown(&self.unknown)
            .build()
    }
}

fn missing(key: &str) -> YamlError {
    YamlError(format!("missing property '{key}'"))
}
