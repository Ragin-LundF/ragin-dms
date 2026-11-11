//! The open comparison of "Convert to real PDF" (Kotlin `RebuildController.compare`): one at a time, until the user
//! keeps the original, the rebuild or both.

use std::sync::Mutex;

use dms_application::port::pdf::RebuiltFile;
use dms_domain::document::DocumentId;
use dms_domain::library::RelativePath;

use crate::session::lock;

#[derive(Debug, Clone)]
pub struct Comparison {
    pub document_id: DocumentId,
    pub path: RelativePath,
    pub rebuilt: RebuiltFile,
    pub original_size: u64,
}

// shortcut: process-wide like the Kotlin controller (one window); move into AppContext if a second window comes.
static COMPARISON: Mutex<Option<Comparison>> = Mutex::new(None);

pub fn comparison() -> Option<Comparison> {
    lock(&COMPARISON).clone()
}

pub fn set_comparison(comparison: Option<Comparison>) {
    *lock(&COMPARISON) = comparison;
}

/// The rebuilt file of the open comparison with this id, for the `dms://` route that serves its page pictures.
pub fn rebuilt_file(rebuild_id: &str) -> Option<RebuiltFile> {
    lock(&COMPARISON).as_ref().filter(|open| open.rebuilt.id == rebuild_id).map(|open| open.rebuilt.clone())
}

/// Forgets the open comparison when another library opens; returns it so its rebuilt file can be discarded.
pub fn take_comparison() -> Option<Comparison> {
    lock(&COMPARISON).take()
}
