//! What Settings › Archive health and the dashboard card show.

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::dashboard::dto::TaskRowDto;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum HealthScheduleDto {
    Off,
    Weekly,
    Monthly,
}

/// A state's colour; the text beside it always names the state too.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum HealthToneDto {
    Good,
    Warning,
    Error,
    Neutral,
}

/// The archive health of the open library on this device.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HealthStateDto {
    /// "All intact", "2 problems found", "Not checked yet".
    pub headline: String,
    pub tone: HealthToneDto,
    /// Files read and compared in the current or last pass.
    pub files_checked: u32,
    /// Share of the pass done, 0 to 1.
    pub done: f32,
    /// When the last complete check ended (RFC 3339).
    pub last_check_at: Option<String>,
    /// A check stopped midway; "Check now" continues it.
    pub unfinished: bool,
    pub schedule: HealthScheduleDto,
    pub folders: Vec<HealthFolderDto>,
    /// The open findings as task rows, with their buttons (repair, dismiss).
    pub findings: Vec<TaskRowDto>,
    pub snapshots: Vec<HealthSnapshotDto>,
}

/// One library folder: what the last pass read there and its state as a pill.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HealthFolderDto {
    pub name: String,
    pub location: String,
    pub primary: bool,
    /// "2,341 files checked · 3 not downloaded, skipped".
    pub summary: String,
    pub pill: String,
    pub tone: HealthToneDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HealthSnapshotDto {
    pub id: String,
    /// RFC 3339.
    pub taken_at: String,
    /// "Revision 42".
    pub label: String,
    /// The snapshot of the current catalog cannot be restored; it is the catalog.
    pub current: bool,
}

/// What restoring a snapshot would change, one line per part ("1 document comes back").
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HealthSnapshotDiffDto {
    pub lines: Vec<String>,
}
