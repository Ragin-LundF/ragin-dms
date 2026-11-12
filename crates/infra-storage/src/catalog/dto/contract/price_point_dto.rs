use crate::catalog::dto::contract::ContractPriceDto;
use crate::catalog::dto::owned;
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// A price valid from a day on, and the document that announced it. `NON_NULL`.
#[derive(Debug, Clone, PartialEq)]
pub struct PricePointDto {
    pub valid_from: String,
    pub price: ContractPriceDto,
    pub document_id: Option<String>,
}

impl PricePointDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        let price = fields.get("price").ok_or_else(|| YamlError("missing property 'price'".to_owned()))?;
        Ok(Self {
            valid_from: fields.text("validFrom")?.to_owned(),
            price: ContractPriceDto::read(price)?,
            document_id: owned(fields.opt_text("documentId")?),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .text("validFrom", &self.valid_from)
            .node("price", self.price.write())
            .opt_text("documentId", self.document_id.as_deref())
            .build()
    }
}
