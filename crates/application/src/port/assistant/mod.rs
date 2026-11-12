//! The AI reading one document, answering about the whole library, drafting a letter, reading a contract's terms,
//! splitting a scanned stack, or turning a sentence into filters.

mod contract_terms_reader;
mod document_assistant;
mod letter_writer;
mod library_assistant;
pub mod model;
mod query_translator;
mod split_proposer;

pub use contract_terms_reader::ContractTermsReader;
pub use document_assistant::DocumentAssistant;
pub use letter_writer::LetterWriter;
pub use library_assistant::LibraryAssistant;
pub use model::{
    AssistantAnswer, AssistantRequest, ChatRole, ChatTurn, ContractTermsAnswer, ContractTermsReading,
    ContractTermsRequest, LetterAnswer, LetterDraft, LetterKind, LetterLanguage, LetterParty, LetterRequest, ReadPrice,
    SourceChunk,
};
pub use model::{FieldAnswer, QueryAnswer, QueryRequest, SplitAnswer, SplitPage, SplitRequest, TranslatedQuery};
pub use query_translator::QueryTranslator;
pub use split_proposer::SplitProposer;
