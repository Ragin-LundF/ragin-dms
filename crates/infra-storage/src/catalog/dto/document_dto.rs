use indexmap::IndexMap;

use crate::catalog::dto::tax::TaxAssignmentDto;
use crate::catalog::dto::{ClassificationDto, FieldValueDto, RetentionDto};
use crate::yaml::{Fields, MapWriter, Node, YamlError};

use super::{extensible, owned, owned_texts, texts_node};

/// One document. Dates are ISO-8601 strings, so the file reads the same on every device. `tax` was added in 2.1.
/// `NON_EMPTY`.
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentDto {
    pub id: String,
    pub path: String,
    pub content_hash: String,
    pub size: i64,
    pub added_at: String,
    pub title: Option<String>,
    pub document_date: Option<String>,
    pub correspondent: Option<String>,
    pub category_id: Option<String>,
    pub tags: Vec<String>,
    pub classification: Option<ClassificationDto>,
    pub fields: IndexMap<String, FieldValueDto>,
    pub note: Option<String>,
    pub retention: Option<RetentionDto>,
    pub tax: Option<TaxAssignmentDto>,
    pub unknown: IndexMap<String, Node>,
}

extensible!(DocumentDto);

const KNOWN: [&str; 15] = [
    "id",
    "path",
    "contentHash",
    "size",
    "addedAt",
    "title",
    "documentDate",
    "correspondent",
    "categoryId",
    "tags",
    "fields",
    "note",
    "retention",
    "classification",
    "tax",
];

impl DocumentDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            id: fields.text("id")?.to_owned(),
            path: fields.text("path")?.to_owned(),
            content_hash: fields.text("contentHash")?.to_owned(),
            size: fields.i64("size")?,
            added_at: fields.text("addedAt")?.to_owned(),
            title: owned(fields.opt_text("title")?),
            document_date: owned(fields.opt_text("documentDate")?),
            correspondent: owned(fields.opt_text("correspondent")?),
            category_id: owned(fields.opt_text("categoryId")?),
            tags: owned_texts(fields, "tags")?,
            classification: fields.get("classification").map(ClassificationDto::read).transpose()?,
            fields: read_field_values(fields, "fields")?,
            note: owned(fields.opt_text("note")?),
            retention: fields.get("retention").map(RetentionDto::read).transpose()?,
            tax: fields.get("tax").map(TaxAssignmentDto::read).transpose()?,
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .non_empty_text("id", Some(&self.id))
            .non_empty_text("path", Some(&self.path))
            .non_empty_text("contentHash", Some(&self.content_hash))
            .int("size", self.size)
            .non_empty_text("addedAt", Some(&self.added_at))
            .non_empty_text("title", self.title.as_deref())
            .non_empty_text("documentDate", self.document_date.as_deref())
            .non_empty_text("correspondent", self.correspondent.as_deref())
            .non_empty_text("categoryId", self.category_id.as_deref())
            .non_empty_list("tags", texts_node(&self.tags))
            .non_empty_map("fields", write_field_values(&self.fields))
            .non_empty_text("note", self.note.as_deref())
            .opt_node("retention", self.retention.as_ref().map(RetentionDto::write))
            .opt_node("tax", self.tax.as_ref().map(TaxAssignmentDto::write))
            .opt_node("classification", self.classification.as_ref().map(ClassificationDto::write))
            .unknown(&self.unknown)
            .build()
    }
}

/// A mapping of field key to value; missing or null is none.
pub(crate) fn read_field_values(fields: Fields<'_>, key: &str) -> Result<IndexMap<String, FieldValueDto>, YamlError> {
    fields.entries(key)?.into_iter().map(|(key, value)| Ok((key.to_owned(), FieldValueDto::read(value)?))).collect()
}

pub(crate) fn write_field_values(values: &IndexMap<String, FieldValueDto>) -> IndexMap<String, Node> {
    values.iter().map(|(key, value)| (key.clone(), value.write())).collect()
}
