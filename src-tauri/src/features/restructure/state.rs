//! What the restructure screen keeps between commands, as Kotlin's view model did: the plan last previewed (until it
//! is applied) and the folders the last run left empty (until the user answers the offer to remove them).

use std::sync::Mutex;

use dms_domain::restructure::RestructurePlan;

use crate::session::lock;

// shortcut: process-wide like the Kotlin view model (one window); move into AppContext if a second window comes.
static PLAN: Mutex<Option<RestructurePlan>> = Mutex::new(None);
static EMPTY_FOLDERS: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub fn plan() -> Option<RestructurePlan> {
    lock(&PLAN).clone()
}

pub fn set_plan(plan: Option<RestructurePlan>) {
    *lock(&PLAN) = plan;
}

pub fn empty_folders() -> Vec<String> {
    lock(&EMPTY_FOLDERS).clone()
}

pub fn set_empty_folders(folders: Vec<String>) {
    *lock(&EMPTY_FOLDERS) = folders;
}

/// Forgets the previewed plan and the offered empty folders when another library opens.
pub fn reset() {
    set_plan(None);
    set_empty_folders(Vec::new());
}
