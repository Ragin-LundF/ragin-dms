use dms_domain::deadline::UpcomingDate;
use dms_domain::task::OpenTask;

use crate::library::FolderState;
use crate::port::state::ActivityEntry;

/// Everything the dashboard shows; `upcoming` are the next deadline and retention dates.
#[derive(Debug, Clone, PartialEq)]
pub struct Dashboard {
    pub tasks: Vec<OpenTask>,
    pub folders: Vec<FolderState>,
    pub document_count: usize,
    pub uncategorized_count: usize,
    pub recent_activity: Vec<ActivityEntry>,
    pub upcoming: Vec<UpcomingDate>,
}
