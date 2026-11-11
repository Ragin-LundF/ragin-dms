//! The HTTP side: OpenAI-compatible chat completions and Anthropic's Messages API, the request bodies, the model
//! choice and cleaning the answers.

mod answer_text;
mod anthropic_request;
mod chat_reply;
pub(crate) mod chat_request;
mod chat_transport;
pub(crate) mod dto;

pub(crate) use answer_text::AnswerText;
pub(crate) use chat_reply::{ChatOptions, ChatReply};
pub use chat_transport::ANTHROPIC_KEY_VARIABLE;
pub(crate) use chat_transport::{ChatTransport, failure_of, local_base_of, request_timeout, shortened};

#[cfg(test)]
mod tests;
