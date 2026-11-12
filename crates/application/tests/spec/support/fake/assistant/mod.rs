//! Fakes of the assistant ports that `TestLibrary` does not hold, owned by the letter, contract, split and search
//! tests.

mod fake_contract_terms_reader;
mod fake_letter_writer;
mod fake_query_translator;
mod fake_split_proposer;

pub use fake_contract_terms_reader::FakeContractTermsReader;
pub use fake_letter_writer::FakeLetterWriter;
pub use fake_query_translator::FakeQueryTranslator;
pub use fake_split_proposer::FakeSplitProposer;
