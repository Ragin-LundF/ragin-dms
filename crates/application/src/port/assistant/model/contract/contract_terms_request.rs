/// What the AI needs to read a contract's terms: its name and correspondent, and the texts of its documents, already
/// cut to the AI's input budget and numbered from 1 in this order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractTermsRequest {
    pub contract_name: String,
    pub correspondent: Option<String>,
    pub source_texts: Vec<String>,
}
