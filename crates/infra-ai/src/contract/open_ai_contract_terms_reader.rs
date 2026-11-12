use std::sync::Arc;

use dms_application::port::ai::ClassificationFailure;
use dms_application::port::assistant::{ContractTermsAnswer, ContractTermsReader, ContractTermsRequest};

use super::contract_prompts;
use super::contract_terms_dto::contract_terms_schema;
use super::contract_terms_mapper::reading_of;
use crate::client::chat_request::{self, ChatMessage};
use crate::client::{ChatOptions, ChatReply, ChatTransport};

const TEMPERATURE: f64 = 0.0;
const SCHEMA_NAME: &str = "contract_terms";
const NO_TERMS: &str = "The answer is no reading of the contract";

/// Reads a contract's prices and terms on the chat model, constrained by the contract terms schema. Neither the
/// documents nor the answer are logged.
pub struct OpenAiContractTermsReader {
    transport: Arc<ChatTransport>,
}

impl OpenAiContractTermsReader {
    pub(crate) fn new(transport: Arc<ChatTransport>) -> Self {
        Self { transport }
    }
}

impl ContractTermsReader for OpenAiContractTermsReader {
    fn read(&self, request: &ContractTermsRequest) -> ContractTermsAnswer {
        let messages =
            [ChatMessage::system(&contract_prompts::system()), ChatMessage::user(&contract_prompts::request(request))];
        let options = ChatOptions { structured: true, ..ChatOptions::default() };
        let reply = self.transport.complete(options, |model| {
            chat_request::structured(model, TEMPERATURE, &messages, SCHEMA_NAME, &contract_terms_schema())
        });
        match reply {
            ChatReply::Failed { reason, detail } => ContractTermsAnswer::Failed { reason, detail },
            ChatReply::Answered { content, model } => match reading_of(&content) {
                Some(reading) => ContractTermsAnswer::Read { reading, model },
                None => ContractTermsAnswer::Failed {
                    reason: ClassificationFailure::InvalidResponse,
                    detail: Some(NO_TERMS.to_owned()),
                },
            },
        }
    }
}
