use indexmap::IndexMap;

use crate::catalog::dto::tax::TaxAssignmentDto;
use crate::catalog::dto::{
    FieldValueDto, owned, owned_texts, read_field_values, read_list, texts_node, write_field_values,
};
use crate::state::dto::SuggestedTagDto;
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// A pending AI suggestion in `tasks.yaml`. A suggested category is either `existing_category_id` or
/// `new_category_name`. `NON_EMPTY`. `fields` and `eInvoice` (the parts read from an e-invoice: `correspondent`,
/// `documentDate`, `field:<key>`) are 2.1 additions.
#[derive(Debug, Clone, PartialEq)]
pub struct SuggestionDto {
    pub document_id: String,
    pub model: String,
    pub title: Option<String>,
    pub existing_category_id: Option<String>,
    pub new_category_name: Option<String>,
    pub new_category_parent_id: Option<String>,
    pub tags: Vec<SuggestedTagDto>,
    pub document_date: Option<String>,
    pub correspondent: Option<String>,
    pub file_name: Option<String>,
    pub transcription: Option<String>,
    pub confidence: Option<f64>,
    /// The suggested tax year and area (2.1); left out when there is none.
    pub tax: Option<TaxAssignmentDto>,
    pub fields: IndexMap<String, FieldValueDto>,
    pub e_invoice: Vec<String>,
    pub requested_actions: Vec<RequestedActionDto>,
}

impl SuggestionDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            document_id: fields.text("documentId")?.to_owned(),
            model: fields.text("model")?.to_owned(),
            title: owned(fields.opt_text("title")?),
            existing_category_id: owned(fields.opt_text("existingCategoryId")?),
            new_category_name: owned(fields.opt_text("newCategoryName")?),
            new_category_parent_id: owned(fields.opt_text("newCategoryParentId")?),
            tags: read_list(fields, "tags", SuggestedTagDto::read)?,
            document_date: owned(fields.opt_text("documentDate")?),
            correspondent: owned(fields.opt_text("correspondent")?),
            file_name: owned(fields.opt_text("fileName")?),
            transcription: owned(fields.opt_text("transcription")?),
            confidence: fields.opt_f64("confidence")?,
            tax: fields.get("tax").map(TaxAssignmentDto::read).transpose()?,
            fields: read_field_values(fields, "fields")?,
            e_invoice: owned_texts(fields, "eInvoice")?,
            requested_actions: read_list(fields, "requestedActions", RequestedActionDto::read)?,
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .non_empty_text("documentId", Some(&self.document_id))
            .non_empty_text("model", Some(&self.model))
            .non_empty_text("title", self.title.as_deref())
            .non_empty_text("existingCategoryId", self.existing_category_id.as_deref())
            .non_empty_text("newCategoryName", self.new_category_name.as_deref())
            .non_empty_text("newCategoryParentId", self.new_category_parent_id.as_deref())
            .non_empty_list("tags", self.tags.iter().map(SuggestedTagDto::write).collect())
            .non_empty_text("documentDate", self.document_date.as_deref())
            .non_empty_text("correspondent", self.correspondent.as_deref())
            .non_empty_text("fileName", self.file_name.as_deref())
            .non_empty_text("transcription", self.transcription.as_deref())
            .opt_node("confidence", self.confidence.map(Node::float))
            .opt_node("tax", self.tax.as_ref().map(TaxAssignmentDto::write))
            .non_empty_map("fields", write_field_values(&self.fields))
            .non_empty_list("eInvoice", texts_node(&self.e_invoice))
            .non_empty_list("requestedActions", self.requested_actions.iter().map(RequestedActionDto::write).collect())
            .build()
    }
}

/// Something the letter asks the person to do; new in 2.1, written only when a suggestion has one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestedActionDto {
    pub text: String,
    pub due_date: Option<String>,
    pub references: Vec<String>,
}

impl RequestedActionDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            text: fields.text("text")?.to_owned(),
            due_date: owned(fields.opt_text("dueDate")?),
            references: fields.texts("references")?.into_iter().map(str::to_owned).collect(),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .text("text", &self.text)
            .non_empty_text("dueDate", self.due_date.as_deref())
            .non_empty_list("references", self.references.iter().map(Node::text).collect())
            .build()
    }
}
