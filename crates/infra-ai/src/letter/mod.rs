//! Drafting letters from the facts a feature knows and the texts of the source documents.

mod letter_answer_dto;
mod letter_prompts;
mod open_ai_letter_writer;

pub use open_ai_letter_writer::OpenAiLetterWriter;

#[cfg(test)]
mod tests;
