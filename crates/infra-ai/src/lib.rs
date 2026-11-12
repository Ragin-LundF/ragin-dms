//! The AI adapters of ragin-dms, ported from the Kotlin module `dms-ai`: the classifier, the document and library
//! assistants, the letter writer, the contract terms reader, the split proposer, the query translator, embeddings and
//! the page layout reader, on any server with the OpenAI chat completions API (LM Studio by default) or on Anthropic's Messages API. Blocking HTTP with `ureq`; every request has a timeout.
//!
//! The Anthropic API key is read from `ANTHROPIC_API_KEY` only, never stored or logged, and sent only to
//! `api.anthropic.com`. Embeddings always use the local server. Nothing here logs document text or prompts.

mod ai_adapters;
mod assistant;
mod classify;
mod client;
mod contract;
mod embedding;
mod letter;
mod prompt;
mod query;
mod rebuild;
mod split;
#[cfg(test)]
mod test_support;

pub use ai_adapters::{AiAdapters, KeySource, SettingsSource};
pub use assistant::{OpenAiAssistant, OpenAiLibraryAssistant};
pub use classify::OpenAiCompatibleClient;
pub use client::ANTHROPIC_KEY_VARIABLE;
pub use contract::OpenAiContractTermsReader;
pub use embedding::OpenAiEmbeddings;
pub use letter::OpenAiLetterWriter;
pub use query::OpenAiQueryTranslator;
pub use rebuild::AiPageLayoutReader;
pub use split::OpenAiSplitProposer;
