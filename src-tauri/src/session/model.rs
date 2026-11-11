use serde::{Deserialize, Serialize};
use specta::Type;

/// Operations that must not run at the same time share a lane; operations in different lanes run side by side.
/// `Library` holds everything that changes files or the catalog, so a scan never sees a move half done. `Ai`
/// holds the AI's runs, which only add suggestions and use one AI server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Lane {
    Library,
    Ai,
}

/// An operation that runs now, in its `lane`; `cancellable` tells whether the user may stop it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RunningOperation {
    pub label: String,
    pub lane: Lane,
    pub cancellable: bool,
}

/// How far the running operation is: `done` of `total` steps, `total` is `None` while it is not known yet.
/// `step` names the current stage, `detail` the item being worked on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OperationProgress {
    pub step: String,
    pub done: u32,
    pub total: Option<u32>,
    pub detail: Option<String>,
}

impl OperationProgress {
    pub fn new(step: impl Into<String>, done: usize, total: Option<usize>) -> Self {
        let count = |value: usize| u32::try_from(value).unwrap_or(u32::MAX);
        Self { step: step.into(), done: count(done), total: total.map(count), detail: None }
    }

    #[must_use]
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// Whether a library is open, and why not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LibraryStatus {
    Loading,
    /// No primary folder is configured on this device yet.
    NotConfigured,
    /// The library is open. `foreignFolders` hold another library and are ignored; `unreachableFolders` could not
    /// be read when it was opened (folder ids).
    #[serde(rename_all = "camelCase")]
    Open {
        foreign_folders: Vec<String>,
        unreachable_folders: Vec<String>,
    },
    /// The primary folder or its catalog could not be read; `reason` is already phrased for the user.
    #[serde(rename_all = "camelCase")]
    PrimaryFailed {
        reason: String,
        detail: Option<String>,
    },
    /// Opening failed unexpectedly, for example because the settings file is unreadable.
    #[serde(rename_all = "camelCase")]
    Failed {
        detail: Option<String>,
    },
}

/// One configured library folder as the shell shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SessionFolder {
    pub id: String,
    pub location: String,
    pub primary: bool,
}

/// The state every screen shares. `revision` counts finished operations and the changes a running operation
/// reports, so screens know when to reload what they show.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SessionState {
    pub status: LibraryStatus,
    pub folders: Vec<SessionFolder>,
    /// Folders the last scan could not read (ids).
    pub offline_folders: Vec<String>,
    /// When the last scan finished, RFC 3339.
    pub last_scan_at: Option<String>,
    pub operations: Vec<RunningOperation>,
    pub ai_enabled: bool,
    pub add_new_files_while_scanning: bool,
    pub revision: u32,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            status: LibraryStatus::Loading,
            folders: Vec::new(),
            offline_folders: Vec::new(),
            last_scan_at: None,
            operations: Vec::new(),
            ai_enabled: false,
            add_new_files_while_scanning: true,
            revision: 0,
        }
    }
}

impl SessionState {
    /// The label of the operation running in `lane`, or `None` when the lane is free.
    pub fn busy_in(&self, lane: Lane) -> Option<&str> {
        self.operations.iter().find(|operation| operation.lane == lane).map(|operation| operation.label.as_str())
    }
}

/// A short message for the status line, such as the result of an action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UiMessage {
    pub text: String,
    pub is_error: bool,
}

impl UiMessage {
    pub fn info(text: impl Into<String>) -> Self {
        Self { text: text.into(), is_error: false }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self { text: text.into(), is_error: true }
    }
}

/// One line of the session log; `at` is RFC 3339.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub at: String,
    pub text: String,
    pub is_error: bool,
}
