//! Reading a contract's prices, notice period and term end from its documents.

mod contract_prompts;
mod contract_terms_dto;
mod contract_terms_mapper;
mod open_ai_contract_terms_reader;

pub use open_ai_contract_terms_reader::OpenAiContractTermsReader;

#[cfg(test)]
mod tests;
