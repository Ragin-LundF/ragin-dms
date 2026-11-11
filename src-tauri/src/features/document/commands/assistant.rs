//! The assistant's commands (Kotlin `SummaryController`, `ChatController`): they only read, so they run beside the
//! lanes. The window drops an answer that arrives after the user closed or stopped the dialog.

use std::sync::Arc;

use dms_application::assistant::AnswerOutcome;
use dms_application::port::assistant::{ChatRole, ChatTurn};

use super::wiring;
use crate::commands::AppState;
use crate::dto::IpcError;
use crate::features::document::dto::{ChatAnswer, ChatMessage, ChatSpeaker, SummaryAnswer};
use crate::features::document::view::texts::answer_problem;
use crate::session::reporting::blocking;

/// The AI's summary of `document_id`.
#[tauri::command]
#[specta::specta]
pub async fn summarize_document(ctx: AppState<'_>, document_id: String) -> Result<SummaryAnswer, IpcError> {
    let ctx = Arc::clone(&ctx);
    blocking(move || {
        let catalog = ctx.require_catalog()?;
        let document = wiring::find_document(&catalog, &document_id)?;
        let outcome = wiring::summarize(&ctx, &document.id)?;
        let problem = answer_problem(&outcome, document.path.file_name());
        Ok(match outcome {
            AnswerOutcome::Answered { text, model } => SummaryAnswer { text: Some(text), model: Some(model), problem },
            _ => SummaryAnswer { text: None, model: None, problem },
        })
    })
    .await
}

/// The AI's answer to the last question of `conversation` about `document_id`; the whole conversation is sent.
#[tauri::command]
#[specta::specta]
pub async fn ask_about_document(
    ctx: AppState<'_>,
    document_id: String,
    conversation: Vec<ChatMessage>,
) -> Result<ChatAnswer, IpcError> {
    let ctx = Arc::clone(&ctx);
    blocking(move || {
        let catalog = ctx.require_catalog()?;
        let document = wiring::find_document(&catalog, &document_id)?;
        let turns: Vec<ChatTurn> = conversation.into_iter().map(turn_of).collect();
        let outcome = wiring::ask(&ctx, &document.id, &turns)?;
        let problem = answer_problem(&outcome, document.path.file_name());
        Ok(match outcome {
            AnswerOutcome::Answered { text, .. } => ChatAnswer { text: Some(text), problem },
            _ => ChatAnswer { text: None, problem },
        })
    })
    .await
}

fn turn_of(message: ChatMessage) -> ChatTurn {
    let role = match message.role {
        ChatSpeaker::User => ChatRole::User,
        ChatSpeaker::Assistant => ChatRole::Assistant,
    };
    ChatTurn { role, text: message.text }
}
