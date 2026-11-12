use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// An amount with its currency, written like a money field value: `{ amount: "12.30", currency: EUR }`.
#[derive(Debug, Clone, PartialEq)]
pub struct MoneyDto {
    pub amount: String,
    pub currency: String,
}

impl MoneyDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self { amount: fields.text("amount")?.to_owned(), currency: fields.text("currency")?.to_owned() })
    }

    pub fn write(&self) -> Node {
        MapWriter::new().text("amount", &self.amount).text("currency", &self.currency).build()
    }
}
