use crate::catalog::dto::{owned, read_list};
use crate::state::dto::SuggestedTagDto;
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// A pending AI suggestion in `tasks.yaml`. A suggested category is either `existing_category_id` or
/// `new_category_name`. `NON_EMPTY`.
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
            .build()
    }
}
