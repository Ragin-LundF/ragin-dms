use std::sync::Arc;

use crate::port::ai::{AiServer, ModelList};
use crate::port::state::AiSettings;

/// Asks the AI server which models it offers; the settings use it as their connection test.
pub struct ListAiModels {
    server: Arc<dyn AiServer>,
}

impl ListAiModels {
    pub fn new(server: Arc<dyn AiServer>) -> Self {
        Self { server }
    }

    pub fn run(&self, settings: &AiSettings) -> ModelList {
        self.server.models(settings)
    }
}
