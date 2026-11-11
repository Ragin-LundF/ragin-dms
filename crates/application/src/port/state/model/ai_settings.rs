use std::ops::RangeInclusive;

use dms_domain::DomainError;

use crate::port::state::AiProvider;

/// Where the AI runs and how it is used, per device. The default is LM Studio's local server. [`Self::model`]
/// `None` uses the first model the server has loaded. [`Self::prefer_vision_for_pdfs`] sends rendered pages of a
/// PDF even when it has a text layer, which helps with poor scanner OCR but is slower.
/// [`Self::accept_suggested_file_names`] checks the suggested file name in a review, so approving renames the
/// file; otherwise the user checks it when the name is better. [`Self::embedding_model`] makes the vectors of the
/// library's index; `None` uses the first embedding model the server offers. [`Self::input_characters`] is how
/// much of a document's text is read and sent to the AI; small local models need less, large ones take more.
/// [`Self::provider`] picks who answers: the local server at [`Self::base_url`] with [`Self::model`], or Anthropic
/// with [`Self::anthropic_model`]. Both are kept, so switching changes nothing else; embeddings always use the
/// local server. [`Self::enabled`] is whether the user set up AI on this device at all; while it is off, the app
/// shows no AI feature.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AiSettings {
    pub enabled: bool,
    pub base_url: String,
    pub model: Option<String>,
    pub prefer_vision_for_pdfs: bool,
    pub timeout_seconds: i32,
    pub accept_suggested_file_names: bool,
    pub embedding_model: Option<String>,
    pub input_characters: i32,
    pub provider: AiProvider,
    pub anthropic_model: String,
}

impl AiSettings {
    pub const DEFAULT_INPUT_CHARACTERS: i32 = 12_000;
    pub const INPUT_CHARACTERS_RANGE: RangeInclusive<i32> = 1_000..=200_000;
    pub const DEFAULT_BASE_URL: &'static str = "http://localhost:1234/v1";
    pub const DEFAULT_TIMEOUT_SECONDS: i32 = 300;
    pub const DEFAULT_ANTHROPIC_MODEL: &'static str = "claude-opus-5-5";

    /// # Errors
    /// Fails for an input budget outside [`Self::INPUT_CHARACTERS_RANGE`].
    pub fn validate(&self) -> Result<(), DomainError> {
        if Self::INPUT_CHARACTERS_RANGE.contains(&self.input_characters) {
            Ok(())
        } else {
            Err(DomainError::InvalidArgument("Input budget must be in 1000..200000".to_owned()))
        }
    }
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: Self::DEFAULT_BASE_URL.to_owned(),
            model: None,
            prefer_vision_for_pdfs: false,
            timeout_seconds: Self::DEFAULT_TIMEOUT_SECONDS,
            accept_suggested_file_names: false,
            embedding_model: None,
            input_characters: Self::DEFAULT_INPUT_CHARACTERS,
            provider: AiProvider::Local,
            anthropic_model: Self::DEFAULT_ANTHROPIC_MODEL.to_owned(),
        }
    }
}
