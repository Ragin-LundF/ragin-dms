use crate::yaml::{Fields, MapWriter, Node, YamlError};

use super::owned;

/// One custom field value. Exactly one of the value properties is set, which also tells its type: `text`,
/// `number`, `amount` with `currency`, `date` (ISO-8601) or `flag`. `NON_NULL`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FieldValueDto {
    pub text: Option<String>,
    pub number: Option<String>,
    pub amount: Option<String>,
    pub currency: Option<String>,
    pub date: Option<String>,
    pub flag: Option<bool>,
}

impl FieldValueDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            text: owned(fields.opt_text("text")?),
            number: owned(fields.opt_text("number")?),
            amount: owned(fields.opt_text("amount")?),
            currency: owned(fields.opt_text("currency")?),
            date: owned(fields.opt_text("date")?),
            flag: fields.opt_bool("flag")?,
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .opt_text("text", self.text.as_deref())
            .opt_text("number", self.number.as_deref())
            .opt_text("amount", self.amount.as_deref())
            .opt_text("currency", self.currency.as_deref())
            .opt_text("date", self.date.as_deref())
            .opt_node("flag", self.flag.map(Node::Bool))
            .build()
    }
}
