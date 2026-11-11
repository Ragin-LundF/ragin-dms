//! Tauri commands: deserialize, call one use case or the session, map to a DTO. Nothing else.

pub mod operations;
pub mod palette;
pub mod session;

use std::sync::Arc;

use tauri::State;

use crate::session::Session;

pub type SessionState<'a> = State<'a, Arc<Session>>;

/// The app context every feature command takes: `ctx: AppState<'_>`.
pub type AppState<'a> = State<'a, Arc<crate::context::AppContext>>;
