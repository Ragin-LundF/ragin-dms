//! Turning a search request in plain words into filters.

mod open_ai_query_translator;
mod query_answer_dto;
mod query_prompts;

pub use open_ai_query_translator::OpenAiQueryTranslator;

#[cfg(test)]
mod tests;
