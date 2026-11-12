use indexmap::IndexMap;

use crate::catalog::dto::contract::{ContractPriceDto, ContractTermsDto, PricePointDto};
use crate::catalog::dto::{extensible, owned, owned_texts, read_list, texts_node};
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// One contract. Terms are left out while empty. `NON_EMPTY`.
#[derive(Debug, Clone, PartialEq)]
pub struct ContractDto {
    pub id: String,
    pub name: String,
    pub correspondent: Option<String>,
    pub kind: String,
    pub price: Option<ContractPriceDto>,
    pub price_history: Vec<PricePointDto>,
    pub terms: Option<ContractTermsDto>,
    pub document_ids: Vec<String>,
    pub ended_on: Option<String>,
    pub note: Option<String>,
    pub unknown: IndexMap<String, Node>,
}

extensible!(ContractDto);

const KNOWN: [&str; 10] =
    ["id", "name", "correspondent", "kind", "price", "priceHistory", "terms", "documentIds", "endedOn", "note"];

impl ContractDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            id: fields.text("id")?.to_owned(),
            name: fields.text("name")?.to_owned(),
            correspondent: owned(fields.opt_text("correspondent")?),
            kind: fields.opt_text("kind")?.unwrap_or("other").to_owned(),
            price: fields.get("price").map(ContractPriceDto::read).transpose()?,
            price_history: read_list(fields, "priceHistory", PricePointDto::read)?,
            terms: fields.get("terms").map(ContractTermsDto::read).transpose()?,
            document_ids: owned_texts(fields, "documentIds")?,
            ended_on: owned(fields.opt_text("endedOn")?),
            note: owned(fields.opt_text("note")?),
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .non_empty_text("id", Some(&self.id))
            .non_empty_text("name", Some(&self.name))
            .non_empty_text("correspondent", self.correspondent.as_deref())
            .non_empty_text("kind", Some(&self.kind))
            .opt_node("price", self.price.as_ref().map(ContractPriceDto::write))
            .non_empty_list("priceHistory", self.price_history.iter().map(PricePointDto::write).collect())
            .opt_node("terms", self.terms.as_ref().map(ContractTermsDto::write))
            .non_empty_list("documentIds", texts_node(&self.document_ids))
            .non_empty_text("endedOn", self.ended_on.as_deref())
            .non_empty_text("note", self.note.as_deref())
            .unknown(&self.unknown)
            .build()
    }
}
