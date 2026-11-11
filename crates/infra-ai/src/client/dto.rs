//! The parts of the servers' answers this crate reads. Unknown keys are ignored; a value of the wrong type makes the
//! whole answer invalid, as in Kotlin.

use serde::Deserialize;

/// The answer of `GET /models`.
#[derive(Debug, Deserialize)]
pub(crate) struct ModelsResponseDto {
    #[serde(default)]
    pub data: Vec<ModelDto>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ModelDto {
    pub id: String,
}

/// An OpenAI-compatible chat completion.
#[derive(Debug, Deserialize)]
pub(crate) struct ChatResponseDto {
    #[serde(default)]
    pub choices: Vec<ChoiceDto>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChoiceDto {
    #[serde(default)]
    pub message: Option<ResponseMessageDto>,
}

/// LM Studio puts a reasoning model's thoughts in `reasoning_content`; with a JSON schema it can put the whole answer
/// there and leave `content` empty.
#[derive(Debug, Deserialize)]
pub(crate) struct ResponseMessageDto {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub reasoning_content: Option<String>,
}

/// An Anthropic Messages API answer.
#[derive(Debug, Deserialize)]
pub(crate) struct MessagesResponseDto {
    #[serde(default)]
    pub content: Vec<ContentBlockDto>,
    #[serde(default)]
    pub stop_reason: Option<String>,
}

/// One block of an Anthropic answer; only `text` blocks carry the answer, thinking blocks are skipped.
#[derive(Debug, Deserialize)]
pub(crate) struct ContentBlockDto {
    #[serde(rename = "type")]
    pub block_type: String,
    #[serde(default)]
    pub text: Option<String>,
}

/// The answer of `POST /embeddings`.
#[derive(Debug, Deserialize)]
pub(crate) struct EmbeddingsResponseDto {
    #[serde(default)]
    pub data: Vec<EmbeddingDto>,
}

/// One vector, at `index` of the texts sent.
#[derive(Debug, Deserialize)]
pub(crate) struct EmbeddingDto {
    #[serde(default)]
    pub index: i64,
    #[serde(default)]
    pub embedding: Vec<f32>,
}
