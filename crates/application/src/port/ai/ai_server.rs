use crate::port::ai::ModelList;
use crate::port::state::AiSettings;

/// The AI server itself: which models it offers, which doubles as the connection test.
pub trait AiServer: Send + Sync {
    fn models(&self, settings: &AiSettings) -> ModelList;
}
