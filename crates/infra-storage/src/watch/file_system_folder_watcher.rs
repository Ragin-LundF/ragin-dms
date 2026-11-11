use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use dms_application::port::storage::FolderWatcher;
use dms_domain::library::{LibraryFolder, RelativePath};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::fs::path_resolver;

const POLL: Duration = Duration::from_millis(500);

/// Watches a folder below every library folder with the platform's watcher (`FSEvents`, inotify,
/// `ReadDirectoryChangesW`) and sends a message on every change. Only a hint: network shares and cloud folders may
/// not report changes at all, which is why the scan, not the watcher, decides what changed.
#[derive(Debug, Clone, Copy, Default)]
pub struct FileSystemFolderWatcher;

impl FolderWatcher for FileSystemFolderWatcher {
    fn changes(&self, folders: &[LibraryFolder], path: &RelativePath) -> Receiver<()> {
        let (sender, receiver) = mpsc::channel();
        let stopped = Arc::new(AtomicBool::new(false));
        let stop = Arc::clone(&stopped);
        let watcher = notify::recommended_watcher(move |event: notify::Result<Event>| {
            if event.is_ok_and(|event| matches!(event.kind, EventKind::Access(_))) {
                return;
            }
            if sender.send(()).is_err() {
                stop.store(true, Ordering::Relaxed);
            }
        });
        let Ok(mut watcher) = watcher else { return receiver };
        let directories = folders
            .iter()
            .filter_map(path_resolver::root)
            .filter_map(|root| path_resolver::existing(&root, path))
            .filter(|directory| directory.is_dir());
        for directory in directories {
            // A folder that cannot be watched is left to the next scan. Unlike Java's WatchService, subfolders
            // created later are watched too; that only adds hints.
            let _ = watcher.watch(&directory, RecursiveMode::Recursive);
        }
        // shortcut: a dropped receiver is noticed at the next change event, so an idle watcher lives until then;
        // a crossbeam channel with disconnect detection would stop it at once.
        thread::spawn(move || {
            let _watcher: RecommendedWatcher = watcher;
            while !stopped.load(Ordering::Relaxed) {
                thread::sleep(POLL);
            }
        });
        receiver
    }
}
