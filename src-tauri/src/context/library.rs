//! The library-wide operations of Kotlin's `LibrarySession` that need the use cases: opening, scanning, the
//! inbox, syncing, the preferences of this device, and following the catalog and the inbox folder.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use dms_application::common::OperationOutcome;
use dms_application::inbox::CollectInboxOutcome;
use dms_application::library::{OpenLibraryOutcome, ScanOutcome, ScanProgress, ScanResult};
use dms_application::port::state::Preferences;
use dms_application::services::AppServices;
use dms_application::sidecar::{FolderMigrationResult, SidecarMigrationOutcome, SidecarMigrationProgress};
use dms_domain::change::DetectedChange;
use dms_domain::library::FolderRole;

use super::AppContext;
use crate::dto::IpcError;
use crate::messages;
use crate::session::reporting::{Reporting, busy};
use crate::session::{Lane, LibraryStatus, Operation, OperationProgress, SessionFolder, UiMessage, lock};

const INBOX_DEBOUNCE: Duration = Duration::from_secs(3);

impl AppContext {
    /// Opens the library, merging the catalogs of all folders, and scans it.
    pub fn open(self: &Arc<Self>) -> Result<(), IpcError> {
        self.run("Opening the library", Lane::Library, false, AppContext::reopen)
    }

    pub fn scan(self: &Arc<Self>) -> Result<(), IpcError> {
        self.run("Scanning the library folders", Lane::Library, true, AppContext::rescan)
    }

    pub fn sync_to_all_destinations(self: &Arc<Self>) -> Result<(), IpcError> {
        self.run("Syncing to all destinations", Lane::Library, false, |ctx, operation| {
            let outcome = ctx.services_or_err()?.sync.sync_from_primary.run();
            operation.session().report(messages::sync(&outcome, "Synced to all destinations"));
            ctx.rescan(operation)
        })
    }

    pub fn merge_all_folders(self: &Arc<Self>) -> Result<(), IpcError> {
        self.run("Merging all folders", Lane::Library, false, |ctx, operation| {
            let outcome = ctx.services_or_err()?.sync.merge_all_folders.run();
            operation.session().report(messages::sync(&outcome, "Merged all folders"));
            ctx.rescan(operation)
        })
    }

    pub fn retry_failed_operations(self: &Arc<Self>) -> Result<(), IpcError> {
        self.run("Retrying failed operations", Lane::Library, false, |ctx, operation| {
            let outcome = ctx.services_or_err()?.sync.retry_failed_operations.run();
            operation.session().report(messages::retry(&outcome));
            Ok(())
        })
    }

    /// Gives documents without a date the one found in their file name or creation date.
    pub fn detect_document_dates(self: &Arc<Self>) -> Result<(), IpcError> {
        self.run("Detecting document dates", Lane::Library, false, |ctx, operation| {
            let dated = ctx.services_or_err()?.library.detect_document_dates.run();
            operation.session().report(UiMessage::info(format!("Found a date for {dated} documents")));
            Ok(())
        })
    }

    /// Lets the AI suggest metadata for every inbox document without one, in the AI lane.
    pub fn analyze_inbox(self: &Arc<Self>) -> Result<(), IpcError> {
        let inbox =
            self.catalog().map(|catalog| catalog.settings().layout.inbox.clone()).ok_or_else(IpcError::not_open)?;
        self.run("Analyzing the inbox", Lane::Ai, true, move |ctx, operation| {
            let progress =
                Reporting::new(operation, |step: &dms_application::classification::ClassificationProgress| {
                    operation.progress(OperationProgress::new("Analyzing the inbox", step.done, Some(step.total)));
                    operation.session().changed();
                });
            ctx.services_or_err()?.ai.classify_all_new.run(&inbox.path, &progress);
            Ok(())
        })
    }

    /// Saves the preferences of this device (Kotlin `updatePreferences` + `PreferencesWriter`): each change is
    /// applied to the newest saved preferences, one at a time, and published to the session.
    pub fn update_preferences(
        self: &Arc<Self>,
        change: impl FnOnce(Preferences) -> Preferences + Send + 'static,
    ) -> Result<(), IpcError> {
        self.run("Saving the preferences", Lane::Library, false, move |ctx, _| {
            ctx.write_preferences(change).map(|_| ())
        })
    }

    /// Opens the library again, for example after its folders changed. Call inside an operation.
    ///
    /// # Errors
    /// A text for the log when the services are missing.
    pub fn reopen(self: &Arc<Self>, operation: &Operation) -> Result<(), String> {
        let services = self.services_or_err()?;
        forget_screen_state(services);
        let folders = services.library.settings.load_library_folders().ok().flatten();
        let status = match services.library.open_library.run() {
            OpenLibraryOutcome::NotConfigured => LibraryStatus::NotConfigured,
            OpenLibraryOutcome::PrimaryFailed(failure) => {
                let detail = failure.detail.clone();
                let reason = messages::failure(failure.reason).to_owned();
                self.session.error(
                    format!("The primary folder failed: {reason} {}", detail.clone().unwrap_or_default())
                        .trim()
                        .to_owned(),
                );
                LibraryStatus::PrimaryFailed { reason, detail }
            }
            OpenLibraryOutcome::Opened { foreign_folders, unreachable_folders, .. } => LibraryStatus::Open {
                foreign_folders: foreign_folders.iter().map(|id| id.value().to_owned()).collect(),
                unreachable_folders: unreachable_folders.iter().map(|id| id.value().to_owned()).collect(),
            },
        };
        let preferences = services.library.settings.load_preferences().unwrap_or_default();
        let open = matches!(status, LibraryStatus::Open { .. });
        self.set_last_scan(None);
        self.session.update(|state| {
            state.status = status;
            state.folders = folders
                .iter()
                .flat_map(dms_domain::library::LibraryFolders::folders)
                .map(|folder| SessionFolder {
                    id: folder.id.value().to_owned(),
                    location: folder.location.value().to_owned(),
                    primary: folder.role == FolderRole::Primary,
                })
                .collect();
            state.offline_folders.clear();
            state.last_scan_at = None;
        });
        self.publish_preferences(&preferences);
        if open {
            self.rescan(operation)?;
        }
        Ok(())
    }

    /// Scans every folder; the dashboard turns the result into tasks. With the preference on, new files join the
    /// library during the scan, and files only the sync folders have are adopted after it. Call inside an operation.
    ///
    /// # Errors
    /// A text for the log when the services are missing.
    pub fn rescan(self: &Arc<Self>, operation: &Operation) -> Result<(), String> {
        let services = self.services_or_err()?;
        if !self.migrate_sidecar_layout(services, operation) {
            return Ok(());
        }
        let adding = self.session.state().add_new_files_while_scanning;
        let reporter = ScanReporter::default();
        let progress = Reporting::new(operation, |step: &ScanProgress| reporter.report(operation, step));
        let result = match services.library.scan_library.run(adding, &progress) {
            ScanOutcome::Scanned(result) => result,
            ScanOutcome::LibraryNotOpen | ScanOutcome::Cancelled => return Ok(()),
        };
        self.session.info(format!("Scan finished: {} open changes", result.changes.len()));
        for folder in result.failed_folders.keys() {
            self.session.error(format!("Folder could not be scanned: {}", folder.value()));
        }
        self.session.update(|state| {
            state.last_scan_at = Some(chrono::Utc::now().to_rfc3339());
            state.offline_folders = result.failed_folders.keys().map(|id| id.value().to_owned()).collect();
        });
        self.set_last_scan(Some(result.clone()));
        self.collect_inbox(services, &result);
        if adding {
            self.adopt_new_files(services, operation, &result);
        }
        Ok(())
    }

    /// Moves the flat 1.x search texts and Ask index of every reachable folder into subfolders before a scan; a
    /// folder already moved is skipped at once, an offline one is moved by a later scan. `false` when cancelled.
    fn migrate_sidecar_layout(&self, services: &AppServices, operation: &Operation) -> bool {
        const STEP: &str = "Moving search texts into subfolders";
        let progress = Reporting::new(operation, |step: &SidecarMigrationProgress| {
            let mut line = OperationProgress::new(STEP, step.done, Some(step.total));
            line.detail = Some(step.folder_name.clone());
            operation.progress(line);
        });
        let (migrations, cancelled) = match services.library.migrate_sidecar_layout.run(&progress) {
            SidecarMigrationOutcome::LibraryNotOpen => return true,
            SidecarMigrationOutcome::Done(migrations) => (migrations, false),
            SidecarMigrationOutcome::Cancelled(migrations) => (migrations, true),
        };
        for migration in &migrations {
            let name = &migration.folder_name;
            match &migration.result {
                FolderMigrationResult::AlreadyCurrent => {}
                FolderMigrationResult::Failed(error) => self.session.info(format!(
                    "Search texts of {name} not moved into subfolders yet: {}",
                    messages::failure(error.reason)
                )),
                FolderMigrationResult::Moved { moved, kept, failed, marked } => {
                    self.session.info(format!("{STEP}: {moved} files moved in {name}"));
                    if !kept.is_empty() {
                        let identical = kept.iter().filter(|kept| kept.identical).count();
                        self.session.info(format!(
                            "{} old files left in place in {name}, their subfolder already has a file ({identical} \
                             with the same content)",
                            kept.len()
                        ));
                    }
                    for (file, error) in failed {
                        self.session.error(format!(
                            "Could not move the file of document {} in {name}: {}",
                            file.document_id.value(),
                            messages::failure(error.reason)
                        ));
                    }
                    if !marked && !cancelled {
                        self.session.info(format!("{name} is moved again at the next scan"));
                    }
                }
            }
        }
        !cancelled
    }

    /// Catalogs what arrived in the inbox of any folder and, with "Analyze automatically" on, lets the AI suggest
    /// metadata for it in the background.
    fn collect_inbox(self: &Arc<Self>, services: &AppServices, result: &ScanResult) {
        let CollectInboxOutcome::Collected(collected) = services.organize.collect_inbox.run(result) else {
            return;
        };
        for change in &collected {
            self.forget(&DetectedChange::NewFile(change.clone()));
        }
        if collected.is_empty() {
            return;
        }
        self.session.info(format!("Inbox: {} new documents", collected.len()));
        let inbox = self.catalog().map(|catalog| catalog.settings().layout.inbox.clone());
        if inbox.is_some_and(|inbox| inbox.auto_analyze && inbox.ai_analysis) && self.session.state().ai_enabled {
            // Busy AI lane: the next scan offers the documents again.
            let _ = self.analyze_inbox();
        }
    }

    fn adopt_new_files(&self, services: &AppServices, operation: &Operation, result: &ScanResult) {
        // A copy of a catalogued document, or of another new file, would only become a second document.
        let mut known: HashSet<_> = self
            .catalog()
            .map(|catalog| catalog.documents().values().map(|document| document.content_hash.clone()).collect())
            .unwrap_or_default();
        let new_files: Vec<_> = result
            .changes
            .iter()
            .filter_map(|change| match change {
                DetectedChange::NewFile(file) => Some(file),
                _ => None,
            })
            .filter(|file| file.content_hash.as_ref().is_some_and(|hash| known.insert(hash.clone())))
            .cloned()
            .collect();
        operation.for_each_with_progress(
            "Adding new files",
            &new_files,
            |file| file.path.value().to_owned(),
            |file| {
                let done = matches!(services.documents.adopt_new_file.run(file), OperationOutcome::Done { .. });
                if done {
                    self.forget(&DetectedChange::NewFile(file.clone()));
                }
                done
            },
        );
    }

    /// Removes a change the user resolved from the last scan, so its task disappears without a new scan.
    pub fn forget(&self, change: &DetectedChange) {
        if let Some(scan) = lock(&self.last_scan).as_mut() {
            scan.changes.retain(|candidate| candidate != change);
        }
        self.session.changed();
    }

    /// Follows the catalog (every screen reloads when it changes) and the inbox folder (a scan a few seconds after
    /// something arrived, while the inbox is on). Call once at start.
    pub fn follow_library(self: &Arc<Self>) {
        let Some(services) = self.services().cloned() else { return };
        let catalogs = services.library.store.subscribe();
        let ctx = Arc::clone(self);
        std::thread::spawn(move || {
            let mut inbox_path = None;
            let mut watch: Option<std::sync::mpsc::Receiver<()>> = None;
            loop {
                match catalogs.recv_timeout(Duration::from_millis(500)) {
                    Ok(catalog) => {
                        ctx.session.changed();
                        let path = catalog.and_then(|catalog| {
                            let inbox = &catalog.settings().layout.inbox;
                            inbox.enabled.then(|| inbox.path.clone())
                        });
                        if path != inbox_path {
                            inbox_path = path;
                            watch = inbox_path.as_ref().and_then(|path| {
                                let folders = services.library.settings.load_library_folders().ok().flatten()?;
                                Some(services.organize.folder_watcher.changes(folders.folders(), path))
                            });
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return,
                }
                if watch.as_ref().is_some_and(|changes| changes.try_recv().is_ok()) {
                    std::thread::sleep(INBOX_DEBOUNCE);
                    while watch.as_ref().is_some_and(|changes| changes.try_recv().is_ok()) {}
                    if matches!(ctx.session.state().status, LibraryStatus::Open { .. }) {
                        let _ = ctx.scan();
                    }
                }
            }
        });
    }

    /// Applies `change` to the newest saved preferences and saves them, one writer at a time (Kotlin
    /// `PreferencesWriter`); every write of this device's preferences goes through here.
    ///
    /// # Errors
    /// The storage failure, phrased for the user.
    pub fn write_preferences(&self, change: impl FnOnce(Preferences) -> Preferences) -> Result<Preferences, String> {
        let settings = &self.services_or_err()?.library.settings;
        let _writer = lock(&self.preferences_writer);
        let current = settings.load_preferences().map_err(|error| messages::failure(error.reason).to_owned())?;
        let saved = change(current);
        settings.save_preferences(&saved).map_err(|error| messages::failure(error.reason).to_owned())?;
        self.publish_preferences(&saved);
        Ok(saved)
    }

    fn publish_preferences(&self, preferences: &Preferences) {
        self.session.update(|state| {
            state.ai_enabled = preferences.ai.enabled;
            state.add_new_files_while_scanning = preferences.add_new_files_while_scanning;
        });
    }

    fn services_or_err(&self) -> Result<&Arc<AppServices>, String> {
        self.services().ok_or_else(|| "the app could not start its services".to_owned())
    }

    /// Runs `job` as the session operation `label`; `busy` when its lane runs another one.
    fn run(
        self: &Arc<Self>,
        label: &str,
        lane: Lane,
        cancellable: bool,
        job: impl FnOnce(&Arc<Self>, &Operation) -> Result<(), String> + Send + 'static,
    ) -> Result<(), IpcError> {
        let ctx = Arc::clone(self);
        if self.session.perform(label, lane, cancellable, move |operation| job(&ctx, operation)) {
            Ok(())
        } else {
            Err(busy(&self.session, lane, label))
        }
    }
}

/// What the restructure, ask and "Convert to real PDF" screens kept from the library that was open before.
fn forget_screen_state(services: &AppServices) {
    crate::features::restructure::state::reset();
    crate::features::ask::state::reset();
    if let Some(comparison) = crate::features::pdf::state::take_comparison() {
        services.pdf.rebuilder.discard(&comparison.rebuilt);
    }
}

/// Turns the progress of a scan into the progress bar and the log (Kotlin `ScanReporter`): one line per folder,
/// its file count, and every file whose content had to be read.
#[derive(Default)]
struct ScanReporter {
    announced: Mutex<Option<ScanProgress>>,
}

impl ScanReporter {
    fn report(&self, operation: &Operation, progress: &ScanProgress) {
        let session = operation.session();
        let folder = format!("{} ({} of {})", progress.folder_name, progress.folder_number, progress.folder_count);
        let mut announced = lock(&self.announced);
        match announced.as_ref() {
            Some(previous) if previous.folder_number == progress.folder_number => {
                if previous.total.is_none() && progress.total.is_some() {
                    session.info(format!(
                        "Found {} files in {}",
                        progress.total.unwrap_or_default(),
                        progress.folder_name
                    ));
                } else if progress.hashed
                    && let Some(current) = &progress.current
                {
                    session.info(format!("Read {}", current.value()));
                }
            }
            _ => session.info(format!("Listing the files of {folder}")),
        }
        *announced = Some(progress.clone());
        let step = if progress.total.is_none() { format!("Listing {folder}") } else { format!("Checking {folder}") };
        let mut line = OperationProgress::new(step, progress.done, progress.total);
        line.detail = progress.current.as_ref().map(|path| path.value().to_owned());
        operation.progress(line);
    }
}
