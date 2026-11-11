//! What a scan found: the differences between the catalog and the library folders.

mod change_detector;
mod detected_change;
pub mod model;

pub use change_detector::ChangeDetector;
pub use detected_change::{ContentConflict, DetectedChange, MissingFile, ModifiedFile, MovedFile, NewFile};
pub use model::{FileFacts, FolderSnapshot};
