//! Splitting a scanned stack: reading its pages with the vision model and proposing where each document starts.

mod open_ai_split_proposer;
mod split_answer_dto;
mod split_prompts;

pub use open_ai_split_proposer::OpenAiSplitProposer;

#[cfg(test)]
mod tests;
