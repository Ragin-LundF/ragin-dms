//! Where the app keeps its device data, and the composition of the services.

use std::path::PathBuf;

/// Overrides the app data folder, so a development run never touches the real library settings.
pub const DATA_DIR_VARIABLE: &str = "RAGIN_DMS_DATA_DIR";

/// The app data folder: `RAGIN_DMS_DATA_DIR` when set, otherwise the platform's folder for `ragin-dms` (as in the
/// Kotlin app, so settings and caches carry over).
pub fn data_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os(DATA_DIR_VARIABLE).filter(|dir| !dir.is_empty()) {
        return Some(PathBuf::from(dir));
    }
    dms_infra_storage::AppDataDirectory::for_application("ragin-dms")
}

mod composition;
mod library;
mod system;

use std::sync::{Arc, Mutex};

use dms_application::library::{Dashboard, DashboardOutcome, ScanResult};
use dms_application::services::AppServices;
use dms_domain::catalog::Catalog;

#[cfg(test)]
pub use composition::ports;
pub use composition::{compose, pdfium_location};

use crate::dto::IpcError;
use crate::session::{Session, lock};

/// What every command reaches: the session and the services of the open library. Managed by Tauri as
/// `Arc<AppContext>`; commands take `State<'_, Arc<AppContext>>`.
pub struct AppContext {
    pub session: Arc<Session>,
    services: Option<Arc<AppServices>>,
    /// The last scan, which the dashboard turns into tasks (Kotlin `SessionStateModel.lastScan`).
    last_scan: Mutex<Option<ScanResult>>,
    /// The dashboard built from the last scan, until the session's revision changes.
    dashboard: Mutex<Option<(u32, Dashboard)>>,
    /// Writes of the preferences run one at a time, each on the newest saved state.
    preferences_writer: Mutex<()>,
}

impl AppContext {
    /// A context without services, when composing failed (the reason is the library status) or in tests.
    pub fn new(session: Arc<Session>) -> Self {
        Self {
            session,
            services: None,
            last_scan: Mutex::new(None),
            dashboard: Mutex::new(None),
            preferences_writer: Mutex::new(()),
        }
    }

    pub fn with_services(session: Arc<Session>, services: AppServices) -> Self {
        Self { services: Some(Arc::new(services)), ..Self::new(session) }
    }

    /// The use cases; `None` only when composing failed at start (`PDFium` missing), which the status shows.
    pub fn services(&self) -> Option<&Arc<AppServices>> {
        self.services.as_ref()
    }

    /// The use cases, or `not_open` for commands that need them.
    ///
    /// # Errors
    /// `not_open` when composing failed at start.
    pub fn require_services(&self) -> Result<&Arc<AppServices>, IpcError> {
        self.services().ok_or_else(IpcError::not_open)
    }

    /// The catalog of the open library; `None` while none is open.
    pub fn catalog(&self) -> Option<Catalog> {
        self.services()?.library.store.catalog()
    }

    /// The catalog, or `not_open` for commands that need one.
    ///
    /// # Errors
    /// `not_open` while no library is open.
    pub fn require_catalog(&self) -> Result<Catalog, IpcError> {
        self.catalog().ok_or_else(IpcError::not_open)
    }

    /// The last scan of this session, if any.
    pub fn last_scan(&self) -> Option<ScanResult> {
        lock(&self.last_scan).clone()
    }

    /// The dashboard of the open library (Kotlin `LibrarySession.dashboard`), built from the last scan once per
    /// session revision; `None` while no library is open.
    pub fn dashboard(&self) -> Option<Dashboard> {
        let revision = self.session.state().revision;
        let mut cached = lock(&self.dashboard);
        if let Some((built, dashboard)) = cached.as_ref()
            && *built == revision
        {
            return Some(dashboard.clone());
        }
        let scan = self.last_scan();
        match self.services()?.library.build_dashboard.run(scan.as_ref()) {
            DashboardOutcome::Built(dashboard) => {
                *cached = Some((revision, dashboard.clone()));
                Some(dashboard)
            }
            DashboardOutcome::LibraryNotOpen => None,
        }
    }

    fn set_last_scan(&self, scan: Option<ScanResult>) {
        *lock(&self.last_scan) = scan;
    }
}
