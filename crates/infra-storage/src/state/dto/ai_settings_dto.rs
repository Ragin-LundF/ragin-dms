use dms_application::port::state::AiSettings;

use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// The AI part of `settings.yaml`. Every field is optional, so older files keep loading. `enabled` is missing in
/// files from before the switch existed; see [`Self::is_enabled`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiSettingsDto {
    pub enabled: Option<bool>,
    pub base_url: String,
    pub model: Option<String>,
    pub prefer_vision_for_pdfs: bool,
    pub timeout_seconds: i32,
    pub accept_suggested_file_names: bool,
    pub embedding_model: Option<String>,
    pub input_characters: i32,
    pub provider: String,
    pub anthropic_model: String,
}

impl Default for AiSettingsDto {
    fn default() -> Self {
        Self {
            enabled: None,
            base_url: AiSettings::DEFAULT_BASE_URL.to_owned(),
            model: None,
            prefer_vision_for_pdfs: false,
            timeout_seconds: AiSettings::DEFAULT_TIMEOUT_SECONDS,
            accept_suggested_file_names: false,
            embedding_model: None,
            input_characters: AiSettings::DEFAULT_INPUT_CHARACTERS,
            provider: "LOCAL".to_owned(),
            anthropic_model: AiSettings::DEFAULT_ANTHROPIC_MODEL.to_owned(),
        }
    }
}

impl AiSettingsDto {
    /// Without the switch, AI counts as set up when any AI setting was changed from its default.
    pub fn is_enabled(&self) -> bool {
        self.enabled.unwrap_or_else(|| *self != Self::default())
    }

    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        let default = Self::default();
        let text_or = |key: &str, default: String| -> Result<String, YamlError> {
            Ok(fields.opt_text(key)?.map_or(default, str::to_owned))
        };
        Ok(Self {
            enabled: fields.opt_bool("enabled")?,
            base_url: text_or("baseUrl", default.base_url)?,
            model: fields.opt_text("model")?.map(str::to_owned),
            prefer_vision_for_pdfs: fields.bool_or("preferVisionForPdfs", default.prefer_vision_for_pdfs)?,
            timeout_seconds: fields.opt_i32("timeoutSeconds")?.unwrap_or(default.timeout_seconds),
            accept_suggested_file_names: fields
                .bool_or("acceptSuggestedFileNames", default.accept_suggested_file_names)?,
            embedding_model: fields.opt_text("embeddingModel")?.map(str::to_owned),
            input_characters: fields.opt_i32("inputCharacters")?.unwrap_or(default.input_characters),
            provider: text_or("provider", default.provider)?,
            anthropic_model: text_or("anthropicModel", default.anthropic_model)?,
        })
    }

    /// Jackson also wrote `isEnabled()` as a property `isEnabled`; it is ignored on reading.
    pub fn write(&self) -> Node {
        MapWriter::new()
            .node("enabled", self.enabled.map_or(Node::Null, Node::Bool))
            .text("baseUrl", &self.base_url)
            .nullable_text("model", self.model.as_deref())
            .bool("preferVisionForPdfs", self.prefer_vision_for_pdfs)
            .int("timeoutSeconds", self.timeout_seconds)
            .bool("acceptSuggestedFileNames", self.accept_suggested_file_names)
            .nullable_text("embeddingModel", self.embedding_model.as_deref())
            .int("inputCharacters", self.input_characters)
            .text("provider", &self.provider)
            .text("anthropicModel", &self.anthropic_model)
            .bool("isEnabled", self.is_enabled())
            .build()
    }
}
