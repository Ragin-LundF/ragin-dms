use dms_domain::library::{LibraryFolderId, RelativePath};
use dms_domain::restructure::PlannedMove;
use indexmap::IndexSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestructureOutcome {
    /// `moved` documents are in their new place in the primary and the catalog; `failed` moves were not possible,
    /// for example because the file is gone or the target appeared in the meantime. `not_catalogued` files did move,
    /// but the catalog could not be saved; a scan shows them, and the record still holds them, so reversing moves
    /// them back. `skipped` documents of a reversal were changed since. `pending_folders` wait in the journal.
    /// `record` names the plan file that can reverse what was done. `empty_folders` were left without documents and
    /// may be removed. A `cancelled` run committed what it had done.
    Applied {
        moved: usize,
        failed: Vec<PlannedMove>,
        not_catalogued: Vec<PlannedMove>,
        skipped: usize,
        pending_folders: IndexSet<LibraryFolderId>,
        record: Option<String>,
        empty_folders: Vec<RelativePath>,
        cancelled: bool,
    },
    LibraryNotOpen,
    /// The plan file to reverse does not exist or cannot be read.
    RecordNotFound,
}
