use std::sync::Arc;

use dms_application::port::state::AiSettings;

use crate::assistant::{OpenAiAssistant, OpenAiLibraryAssistant};
use crate::classify::OpenAiCompatibleClient;
use crate::client::{ANTHROPIC_KEY_VARIABLE, ChatTransport};
use crate::embedding::OpenAiEmbeddings;
use crate::rebuild::AiPageLayoutReader;

/// The AI settings of this device, read for every request, so a change in Settings applies to the next one.
pub type SettingsSource = Arc<dyn Fn() -> AiSettings + Send + Sync>;

/// The Anthropic API key, if there is one.
pub type KeySource = Arc<dyn Fn() -> Option<String> + Send + Sync>;

/// The AI adapters on one shared HTTP client. The classifier also answers [`dms_application::port::ai::AiServer`].
pub struct AiAdapters {
    pub client: OpenAiCompatibleClient,
    pub assistant: OpenAiAssistant,
    pub library_assistant: OpenAiLibraryAssistant,
    pub embeddings: OpenAiEmbeddings,
    pub layout_reader: AiPageLayoutReader,
}

impl AiAdapters {
    /// The adapters with the Anthropic key from the environment variable `ANTHROPIC_API_KEY`.
    #[must_use]
    pub fn new(settings: SettingsSource) -> Self {
        Self::with_anthropic_key(settings, Arc::new(|| std::env::var(ANTHROPIC_KEY_VARIABLE).ok()))
    }

    #[must_use]
    pub fn with_anthropic_key(settings: SettingsSource, anthropic_key: KeySource) -> Self {
        Self::of(ChatTransport::new(settings, anthropic_key))
    }

    pub(crate) fn of(transport: ChatTransport) -> Self {
        let transport = Arc::new(transport);
        Self {
            client: OpenAiCompatibleClient::new(Arc::clone(&transport)),
            assistant: OpenAiAssistant::new(Arc::clone(&transport)),
            library_assistant: OpenAiLibraryAssistant::new(Arc::clone(&transport)),
            embeddings: OpenAiEmbeddings::new(Arc::clone(&transport)),
            layout_reader: AiPageLayoutReader::new(transport),
        }
    }
}
