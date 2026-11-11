//! What the dashboard and the open tasks screen show, and the actions their buttons send back.

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::session::Lane;

/// Everything the dashboard shows (Kotlin `DashboardUiStateModel` plus what its composables derived from it).
/// The tasks screen shows [`Self::groups`].
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DashboardViewDto {
    /// "{n} documents · {size} · {folders} library folders · last scan {dd.MM. HH:mm}".
    pub header_line: String,
    pub stats: Vec<StatColumnDto>,
    /// `None` while AI is off or the AI has nothing to do.
    pub ai_band: Option<AiBandDto>,
    pub attention: AttentionDto,
    pub recent: Vec<RecentDocumentDto>,
    /// The folder thumbnails of catalogued documents are read from.
    pub primary_folder_id: Option<String>,
    /// Largest first, with an "Uncategorized" entry (id `""`) at the end when there are such documents.
    pub categories: Vec<ChartPointDto>,
    pub tags: Vec<ChartPointDto>,
    /// Ids are the years, the first may be "≤YYYY" (that year and all earlier ones).
    pub by_year: Vec<ChartPointDto>,
    pub by_kind: Vec<KindCountDto>,
    pub correspondents: Vec<ChartPointDto>,
    pub folders: Vec<FolderStatusDto>,
    pub activity: Vec<ActivityDto>,
    /// The open tasks by group, in the order of [`TaskGroupKind`]; only groups with tasks.
    pub groups: Vec<TaskGroupDto>,
}

/// One figure of the stat strip. `count` is set when the value is a count the window writes with the
/// grouping separator of the user's locale (Kotlin `%,d`); `value` then holds it without one.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StatColumnDto {
    pub label: String,
    pub value: String,
    pub count: Option<u32>,
    pub caption: String,
    /// A share from 0 to 1 drawn as a meter.
    pub meter: Option<f32>,
    /// Drawn as a sparkline when it has more than one value.
    pub trend: Vec<u32>,
    pub link: Option<StatLink>,
}

/// Where a click on a figure leads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum StatLink {
    Uncategorized,
    Inbox,
}

/// The AI band; a label is `None` while its run has nothing to do. Both buttons wait for the AI lane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AiBandDto {
    pub headline: String,
    pub classify_label: Option<String>,
    pub tag_label: Option<String>,
}

/// "Needs your attention": the most urgent rows (at most five) out of `total`, and the header buttons.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AttentionDto {
    pub rows: Vec<AttentionRowDto>,
    pub total: u32,
    pub header_buttons: Vec<DashboardButtonDto>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum AttentionSeverity {
    Passed,
    Soon,
    Info,
}

/// One row of "Needs your attention". `detail` already starts with the severity's label, so the colour is never
/// the only signal. `icon` names a material icon (snake case).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AttentionRowDto {
    pub key: String,
    pub severity: AttentionSeverity,
    pub title: String,
    pub detail: String,
    pub icon: String,
    pub preview: Option<TaskFileDto>,
    pub buttons: Vec<DashboardButtonDto>,
    pub opens: Vec<DashboardButtonDto>,
    pub copies: Vec<DuplicateCopyDto>,
}

/// A file a row is about: `path` in library folder `folderId`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TaskFileDto {
    pub folder_id: String,
    pub path: String,
}

/// The kinds of open tasks, in the order the dashboard lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum TaskGroupKind {
    Suggestions,
    Deadlines,
    Retention,
    LearnedRules,
    NewFiles,
    Missing,
    Modified,
    Moved,
    Duplicates,
    Conflicts,
    CatalogMerge,
    FailedOperations,
    Restructure,
}

/// The open tasks of one group; `bulk_actions` answer all of them at once.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TaskGroupDto {
    pub group: TaskGroupKind,
    pub label: String,
    /// Material icon of the group, the fallback of a row without a thumbnail.
    pub icon: String,
    pub rows: Vec<TaskRowDto>,
    pub bulk_actions: Vec<DashboardButtonDto>,
}

/// One open task: exactly the actions its use cases support, and `opens` to look at its file. A duplicates task
/// lists its `copies`, each with its own buttons, and then has no `opens`.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TaskRowDto {
    pub key: String,
    pub title: String,
    pub detail: String,
    pub buttons: Vec<DashboardButtonDto>,
    pub opens: Vec<DashboardButtonDto>,
    pub preview: Option<TaskFileDto>,
    pub copies: Vec<DuplicateCopyDto>,
}

/// One copy of a duplicates task.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateCopyDto {
    pub path: String,
    pub detail: String,
    pub open: DashboardButtonDto,
    pub keep: DashboardButtonDto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum DashboardButtonStyle {
    Filled,
    Tonal,
    Outlined,
    Text,
}

/// A computed button. `lane` names the lane whose running operation disables it (with the tooltip `Wait until
/// "{operation}" finishes`); `None` for buttons that only look at something. `confirm` is asked before `action`
/// runs.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DashboardButtonDto {
    pub label: String,
    pub icon: Option<String>,
    pub style: DashboardButtonStyle,
    pub disabled_reason: Option<String>,
    pub lane: Option<Lane>,
    pub action: DashboardAction,
    pub confirm: Option<DashboardConfirmationDto>,
}

/// A question before a destructive action, naming everything it touches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DashboardConfirmationDto {
    pub title: String,
    pub message: String,
    pub affected: Vec<String>,
    pub confirm_label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum MoveChoice {
    /// Use the new name everywhere.
    Apply,
    /// Rename the file back in the folder where it was found.
    Undo,
}

/// What a dashboard button does. The window handles the kinds that only navigate or open something (`review`,
/// `open`, `openDocument`, `openInbox`, `restructure`, `retry`, `findScans`, `exportCalendar`,
/// `makeScansSearchable` through the pdf area's `make_all_searchable`); every other kind
/// goes to `run_dashboard_action`, which finds the task by its key in the current dashboard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum DashboardAction {
    #[serde(rename_all = "camelCase")]
    Review {
        document_id: String,
    },
    #[serde(rename_all = "camelCase")]
    Open {
        folder_id: String,
        path: String,
    },
    #[serde(rename_all = "camelCase")]
    OpenDocument {
        document_id: String,
    },
    OpenInbox,
    Restructure {
        folder: String,
    },
    Retry,
    FindScans,
    ExportCalendar,
    #[serde(rename_all = "camelCase")]
    Reject {
        document_id: String,
    },
    #[serde(rename_all = "camelCase")]
    Adopt {
        task_key: String,
    },
    #[serde(rename_all = "camelCase")]
    TrashNewFile {
        task_key: String,
    },
    #[serde(rename_all = "camelCase")]
    TrashDuplicateFile {
        task_key: String,
    },
    #[serde(rename_all = "camelCase")]
    Restore {
        task_key: String,
    },
    #[serde(rename_all = "camelCase")]
    DeleteEverywhere {
        task_key: String,
    },
    #[serde(rename_all = "camelCase")]
    KeepVersion {
        task_key: String,
        folder_id: String,
    },
    #[serde(rename_all = "camelCase")]
    KeepBoth {
        task_key: String,
        folder_id: String,
    },
    #[serde(rename_all = "camelCase")]
    ResolveMove {
        task_key: String,
        choice: MoveChoice,
    },
    #[serde(rename_all = "camelCase")]
    KeepDuplicate {
        task_key: String,
        document_id: String,
    },
    #[serde(rename_all = "camelCase")]
    Dismiss {
        task_key: String,
    },
    #[serde(rename_all = "camelCase")]
    SettleRetention {
        document_ids: Vec<String>,
    },
    #[serde(rename_all = "camelCase")]
    TrashRetained {
        document_ids: Vec<String>,
    },
    #[serde(rename_all = "camelCase")]
    AcceptRule {
        task_key: String,
    },
    AnalyzeInbox,
    MakeScansSearchable,
    AdoptAll,
    RestoreAll,
    RenameAll,
    RemoveAllDuplicates,
}

/// A recently added document; `subtitle` is "{parent folder or "Top level"} · {category}", `added` the relative
/// date and `added_on` the exact one for its tooltip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RecentDocumentDto {
    pub id: String,
    pub path: String,
    pub title: String,
    pub subtitle: String,
    pub kind: DocumentKindDto,
    pub added: String,
    pub added_on: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum DocumentKindDto {
    Pdf,
    Image,
    Office,
    Other,
}

/// One value of a chart; `id` tells what a click opens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChartPointDto {
    pub id: String,
    pub label: String,
    pub value: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct KindCountDto {
    pub kind: DocumentKindDto,
    pub count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum FolderHealth {
    NotScanned,
    Reachable,
    Offline,
    Foreign,
}

/// One library folder: `title` is "{name}" or "{name} · primary", `status` the health and pending operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FolderStatusDto {
    pub title: String,
    pub location: String,
    pub primary: bool,
    pub health: FolderHealth,
    pub status: String,
}

/// One line of the recent activity: "{verb} {subject}" and when, "dd.MM. HH:mm" in local time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ActivityDto {
    pub line: String,
    pub time: String,
}
