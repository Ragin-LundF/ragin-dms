use dms_application::port::ai::ClassificationFailure;

/// The text the model answered with, from `model`, or why there is none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ChatReply {
    Answered { content: String, model: String },
    Failed { reason: ClassificationFailure, detail: Option<String> },
}

impl ChatReply {
    pub(crate) fn failed(reason: ClassificationFailure, detail: impl Into<String>) -> Self {
        Self::Failed { reason, detail: Some(detail.into()) }
    }
}

/// How a chat request is sent. `has_images` tells a refused request apart: a model without vision answers images
/// with HTTP 400. A `structured` request asks for JSON; its answer may come as the reasoning of a reasoning model
/// when the answer itself is empty. `min_timeout_seconds` lengthens the configured timeout for slow requests.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ChatOptions {
    pub has_images: bool,
    pub structured: bool,
    pub min_timeout_seconds: i32,
}
