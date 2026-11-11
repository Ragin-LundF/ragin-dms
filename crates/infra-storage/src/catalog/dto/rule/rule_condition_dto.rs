use crate::catalog::dto::{owned, owned_texts, texts_node};
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// One rule condition. `condition_type` is `correspondent` (with `name`), `textAny` or `textAll` (with `words`)
/// or `fileName` (with `glob`). `NON_EMPTY`; fields of a newer version are not kept here.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleConditionDto {
    pub condition_type: String,
    pub name: Option<String>,
    pub words: Vec<String>,
    pub glob: Option<String>,
}

impl RuleConditionDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            condition_type: fields.text("type")?.to_owned(),
            name: owned(fields.opt_text("name")?),
            words: owned_texts(fields, "words")?,
            glob: owned(fields.opt_text("glob")?),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .non_empty_text("type", Some(&self.condition_type))
            .non_empty_text("name", self.name.as_deref())
            .non_empty_list("words", texts_node(&self.words))
            .non_empty_text("glob", self.glob.as_deref())
            .build()
    }
}
