use std::io;
use std::time::Duration;

use dms_application::port::ai::{ClassificationFailure, ModelList};
use dms_application::port::state::{AiProvider, AiSettings};
use dms_domain::text;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::anthropic_request::anthropic_body;
use super::dto::{ChatResponseDto, MessagesResponseDto, ModelsResponseDto};
use super::{AnswerText, ChatOptions, ChatReply};
use crate::ai_adapters::{KeySource, SettingsSource};

pub const ANTHROPIC_KEY_VARIABLE: &str = "ANTHROPIC_API_KEY";
/// How much of a refusing server's answer goes into a failure's detail.
pub(crate) const DETAIL_LENGTH: usize = 300;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const MODELS_TIMEOUT: Duration = Duration::from_secs(10);
const ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com/v1";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const REFUSAL: &str = "refusal";
const TEXT_BLOCK: &str = "text";
const NOT_FOR_CHAT: [&str; 5] = ["embed", "rerank", "e5-", "bge-", "gte-"];

/// The HTTP side of any server with the OpenAI chat completions API, LM Studio by default, or of Anthropic's
/// Messages API when the settings choose [`AiProvider::Anthropic`]: the model list, one chat request, and the mapping
/// of errors. The settings are read for every call, so a change applies to the next request. The Anthropic key is
/// sent only to the Anthropic base URL, never to the local server.
pub(crate) struct ChatTransport {
    settings: SettingsSource,
    anthropic_key: KeySource,
    anthropic_base: String,
    agent: ureq::Agent,
}

/// A server's answer: its HTTP status and body.
pub(crate) struct HttpAnswer {
    pub status: u16,
    pub body: String,
}

impl HttpAnswer {
    pub(crate) fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    pub(crate) fn json<T: DeserializeOwned>(&self) -> Result<T, String> {
        serde_json::from_str(&self.body).map_err(|error| error.to_string())
    }
}

impl ChatTransport {
    pub(crate) fn new(settings: SettingsSource, anthropic_key: KeySource) -> Self {
        Self::with_anthropic_base(settings, anthropic_key, ANTHROPIC_BASE_URL)
    }

    /// Only tests point Anthropic requests elsewhere, at their stub server.
    pub(crate) fn with_anthropic_base(settings: SettingsSource, anthropic_key: KeySource, base: &str) -> Self {
        let agent = ureq::Agent::config_builder()
            .timeout_connect(Some(CONNECT_TIMEOUT))
            .http_status_as_error(false)
            .build()
            .into();
        Self { settings, anthropic_key, anthropic_base: base.to_owned(), agent }
    }

    pub(crate) fn settings(&self) -> AiSettings {
        (self.settings)()
    }

    pub(crate) fn models(&self, settings: &AiSettings) -> ModelList {
        let key = match self.key_for(settings) {
            Ok(key) => key,
            Err(detail) => {
                return ModelList::Unavailable {
                    reason: ClassificationFailure::ProviderUnavailable,
                    detail: Some(detail),
                };
            }
        };
        let url = format!("{}/models", self.base_of(settings));
        let answer = match self.send(&url, None, MODELS_TIMEOUT, key.as_deref()) {
            Ok(answer) => answer,
            Err(error) => {
                return ModelList::Unavailable { reason: failure_of(&error), detail: Some(error.to_string()) };
            }
        };
        if !answer.is_success() {
            return ModelList::Unavailable {
                reason: ClassificationFailure::ProviderUnavailable,
                detail: Some(detail_of(&answer)),
            };
        }
        match answer.json::<ModelsResponseDto>() {
            Ok(models) => ModelList::Available(models.data.into_iter().map(|model| model.id).collect()),
            Err(detail) => {
                ModelList::Unavailable { reason: ClassificationFailure::ProviderUnavailable, detail: Some(detail) }
            }
        }
    }

    /// Sends the request `body` builds for the chosen model.
    pub(crate) fn complete(&self, options: ChatOptions, body: impl FnOnce(&str) -> Value) -> ChatReply {
        let settings = self.settings();
        let key = match self.key_for(&settings) {
            Ok(key) => key,
            Err(detail) => return ChatReply::failed(ClassificationFailure::ProviderUnavailable, detail),
        };
        let anthropic = is_anthropic(&settings);
        let model = match self.model_for(&settings) {
            Ok(model) => model,
            Err(failure) => return failure,
        };
        let open_ai = body(&model);
        let (path, request) =
            if anthropic { ("messages", anthropic_body(&open_ai)) } else { ("chat/completions", open_ai) };
        let url = format!("{}/{path}", self.base_of(&settings));
        let timeout = request_timeout(&settings, options.min_timeout_seconds);
        match self.send(&url, Some(&request), timeout, key.as_deref()) {
            Ok(answer) => reply_of(&answer, options, model, anthropic),
            Err(error) => ChatReply::failed(failure_of(&error), error.to_string()),
        }
    }

    /// A GET without `body`, a JSON POST with it. The key is `Some` only for Anthropic requests.
    pub(crate) fn send(
        &self,
        url: &str,
        body: Option<&Value>,
        timeout: Duration,
        anthropic_key: Option<&str>,
    ) -> Result<HttpAnswer, ureq::Error> {
        let mut response = if let Some(body) = body {
            let mut request = self.agent.post(url).config().timeout_global(Some(timeout)).build();
            if let Some(key) = anthropic_key {
                request = request.header("x-api-key", key).header("anthropic-version", ANTHROPIC_VERSION);
            }
            request.header("content-type", "application/json").send(body.to_string())?
        } else {
            let mut request = self.agent.get(url).config().timeout_global(Some(timeout)).build();
            if let Some(key) = anthropic_key {
                request = request.header("x-api-key", key).header("anthropic-version", ANTHROPIC_VERSION);
            }
            request.call()?
        };
        let status = response.status().as_u16();
        let body = response.body_mut().read_to_string()?;
        Ok(HttpAnswer { status, body })
    }

    /// The model named in the settings, or else the first chat model the server lists.
    fn model_for(&self, settings: &AiSettings) -> Result<String, ChatReply> {
        let chosen = if is_anthropic(settings) {
            Some(settings.anthropic_model.clone()).filter(|model| !text::is_blank(model))
        } else {
            settings.model.clone()
        };
        if let Some(model) = chosen {
            return Ok(model);
        }
        match self.models(settings) {
            ModelList::Unavailable { reason, detail } => Err(ChatReply::Failed { reason, detail }),
            ModelList::Available(models) => chat_model_of(&models).ok_or_else(|| {
                ChatReply::failed(ClassificationFailure::ProviderUnavailable, "The AI server has no chat model")
            }),
        }
    }

    /// The Anthropic key for an Anthropic request, `None` for the local server, or the detail of a missing key.
    fn key_for(&self, settings: &AiSettings) -> Result<Option<String>, String> {
        if !is_anthropic(settings) {
            return Ok(None);
        }
        match (self.anthropic_key)() {
            Some(key) if !text::is_blank(&key) => Ok(Some(key)),
            _ => Err(format!("{ANTHROPIC_KEY_VARIABLE} is not set")),
        }
    }

    fn base_of(&self, settings: &AiSettings) -> String {
        if is_anthropic(settings) { self.anthropic_base.clone() } else { local_base_of(settings) }
    }
}

/// The local server's base URL without surrounding spaces and trailing slashes.
pub(crate) fn local_base_of(settings: &AiSettings) -> String {
    text::trim(&settings.base_url).trim_end_matches('/').to_owned()
}

/// The configured timeout, at least `min_seconds`, and at least one second.
pub(crate) fn request_timeout(settings: &AiSettings, min_seconds: i32) -> Duration {
    Duration::from_secs(u64::try_from(settings.timeout_seconds.max(min_seconds)).unwrap_or(0).max(1))
}

fn is_anthropic(settings: &AiSettings) -> bool {
    settings.provider == AiProvider::Anthropic
}

fn reply_of(answer: &HttpAnswer, options: ChatOptions, model: String, anthropic: bool) -> ChatReply {
    if !answer.is_success() {
        // Anthropic answers 400 for many reasons (schema, image size), not for a model without vision.
        let reason = if options.has_images && !anthropic && answer.status == 400 {
            ClassificationFailure::ModelCannotReadImages
        } else {
            ClassificationFailure::ProviderUnavailable
        };
        return ChatReply::failed(reason, detail_of(answer));
    }
    let content = if anthropic { anthropic_content_of(answer) } else { open_ai_content_of(answer, options.structured) };
    match content {
        Err(detail) => ChatReply::failed(ClassificationFailure::InvalidResponse, detail),
        Ok(Some(content)) if !content.is_empty() => ChatReply::Answered { content, model },
        Ok(_) => ChatReply::failed(ClassificationFailure::InvalidResponse, "The AI answered nothing"),
    }
}

fn anthropic_content_of(answer: &HttpAnswer) -> Result<Option<String>, String> {
    let message: MessagesResponseDto = answer.json()?;
    if message.stop_reason.as_deref() == Some(REFUSAL) {
        return Err("The AI declined".to_owned());
    }
    let text: String = message
        .content
        .iter()
        .filter(|block| block.block_type == TEXT_BLOCK)
        .filter_map(|block| block.text.as_deref())
        .collect();
    Ok(Some(AnswerText::clean(&text)))
}

fn open_ai_content_of(answer: &HttpAnswer, structured: bool) -> Result<Option<String>, String> {
    let response: ChatResponseDto = answer.json()?;
    let Some(message) = response.choices.into_iter().next().and_then(|choice| choice.message) else {
        return Ok(None);
    };
    let content = message.content.map(|content| AnswerText::clean(&content)).filter(|content| !content.is_empty());
    let reasoning = || message.reasoning_content.filter(|_| structured).map(|reasoning| AnswerText::clean(&reasoning));
    Ok(content.or_else(reasoning))
}

/// `HTTP <status>: <the start of the answer>`.
pub(crate) fn detail_of(answer: &HttpAnswer) -> String {
    format!("HTTP {}: {}", answer.status, shortened(&answer.body))
}

/// At most [`DETAIL_LENGTH`] UTF-16 units, as Kotlin's `take`.
pub(crate) fn shortened(text: &str) -> String {
    text::take_utf16(text, DETAIL_LENGTH).to_owned()
}

/// A timeout, or an unreachable server for every other failure to get an answer.
pub(crate) fn failure_of(error: &ureq::Error) -> ClassificationFailure {
    let timed_out = match error {
        ureq::Error::Timeout(_) => true,
        ureq::Error::Io(io) => matches!(io.kind(), io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock),
        _ => false,
    };
    if timed_out { ClassificationFailure::Timeout } else { ClassificationFailure::ProviderUnavailable }
}

/// The first model that can chat, when the settings name none. LM Studio can list every downloaded model, including
/// embedding models, which cannot chat; those are skipped.
pub(crate) fn chat_model_of(models: &[String]) -> Option<String> {
    models
        .iter()
        .find(|model| {
            let name = text::lowercase(model);
            !NOT_FOR_CHAT.iter().any(|part| name.contains(part))
        })
        .cloned()
}
