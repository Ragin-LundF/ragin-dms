use chrono::{Days, Months, NaiveDate};

use crate::catalog::Catalog;
use crate::emergency::EmergencyFolder;
use crate::task::OpenTask;

/// The dashboard task that reminds the user to refresh the emergency folder, from 14 days before the refresh is due.
pub struct EmergencyPlanner;

impl EmergencyPlanner {
    const REMIND_DAYS_BEFORE: u64 = 14;

    #[must_use]
    pub fn due_tasks(catalog: &Catalog, today: NaiveDate) -> Vec<OpenTask> {
        let Some(due) = catalog.settings().emergency_folder.as_ref().and_then(Self::next_refresh) else {
            return Vec::new();
        };
        let remind_from = due.checked_sub_days(Days::new(Self::REMIND_DAYS_BEFORE)).unwrap_or(due);
        if today < remind_from {
            return Vec::new();
        }
        vec![OpenTask::EmergencyFolderRefresh { due }]
    }

    /// When the folder is due for its next refresh: `refresh_months` after the last one. `None` for an empty folder
    /// or one that was never refreshed (the screen asks for the first refresh itself).
    #[must_use]
    pub fn next_refresh(folder: &EmergencyFolder) -> Option<NaiveDate> {
        if folder.items.is_empty() {
            return None;
        }
        folder.last_refreshed?.checked_add_months(Months::new(u32::from(folder.refresh_months)))
    }
}
