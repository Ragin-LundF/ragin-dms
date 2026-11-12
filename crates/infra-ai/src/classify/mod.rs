//! Classifying a document: the prompts, the answer schema, the naming convention and mapping the answer to a
//! suggestion.

mod naming_convention;
mod open_ai_compatible_client;
pub(crate) mod prompt_texts;
pub(crate) mod suggestion_dto;
pub(crate) mod suggestion_mapper;
mod suggestion_schema;

pub use open_ai_compatible_client::OpenAiCompatibleClient;

#[cfg(test)]
mod tests;
