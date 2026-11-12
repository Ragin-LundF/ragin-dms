use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// A price per period; the amount is a plain decimal string like the money field values. `NON_NULL`.
#[derive(Debug, Clone, PartialEq)]
pub struct ContractPriceDto {
    pub amount: String,
    pub currency: String,
    pub period: String,
}

impl ContractPriceDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            amount: fields.text("amount")?.to_owned(),
            currency: fields.text("currency")?.to_owned(),
            period: fields.text("period")?.to_owned(),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .text("amount", &self.amount)
            .text("currency", &self.currency)
            .text("period", &self.period)
            .build()
    }
}
