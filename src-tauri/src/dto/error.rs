use serde::{Deserialize, Serialize};
use specta::Type;

/// Why a command failed. Use-case outcomes the user should read are not errors: they arrive as messages or as
/// part of a command's result; this is for broken calls.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct IpcError {
    /// `not_open` (no library is open), `invalid_input`, `busy` (the lane runs another operation), `internal`.
    pub code: String,
    pub message: String,
}

impl IpcError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self { code: code.to_owned(), message: message.into() }
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new("invalid_input", message)
    }

    pub fn not_open() -> Self {
        Self::new("not_open", "No library is open")
    }

    pub fn internal(error: impl std::fmt::Display) -> Self {
        Self::new("internal", error.to_string())
    }
}
