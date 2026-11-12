use dms_domain::health::HealthSchedule;

use crate::port::state::{AiSettings, LayoutPreferences};

/// How the tool behaves on this device. [`Self::add_new_files_while_scanning`] catalogs new files of the primary
/// folder during a scan instead of offering each one on the dashboard. [`Self::ai`] says where the AI runs, and
/// [`Self::layout`] how the library's panels sit, [`Self::health_check`] how often the archive is checked.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Preferences {
    pub add_new_files_while_scanning: bool,
    pub ai: AiSettings,
    pub layout: LayoutPreferences,
    pub health_check: HealthSchedule,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            add_new_files_while_scanning: true,
            ai: AiSettings::default(),
            layout: LayoutPreferences::default(),
            health_check: HealthSchedule::default(),
        }
    }
}
