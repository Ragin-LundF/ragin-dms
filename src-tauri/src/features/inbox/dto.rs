use serde::{Deserialize, Serialize};
use specta::Type;

/// The inbox screen: documents in the inbox folder of any library folder, the folders to file them into and the
/// folders filed into last (this session).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct InboxStateDto {
    pub enabled: bool,
    pub path: String,
    /// The inbox's AI analysis is on and AI is enabled on this device.
    pub ai_analysis: bool,
    pub rows: Vec<InboxRowDto>,
    pub folders: Vec<String>,
    pub recent_targets: Vec<String>,
}

/// One inbox document, newest added first. `subfolder` is where in the inbox it lies ("" directly inside),
/// `suggestion_note` is " · AI suggests {folder}", " · AI suggestion waiting" or "", `prefill` the folder the
/// library's path template gives it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct InboxRowDto {
    pub document_id: String,
    pub name: String,
    pub subfolder: String,
    /// RFC 3339.
    pub added_at: String,
    #[specta(type = f64)]
    pub size: u64,
    pub has_suggestion: bool,
    pub suggestion_note: String,
    pub analyzable: bool,
    pub prefill: Option<String>,
}
