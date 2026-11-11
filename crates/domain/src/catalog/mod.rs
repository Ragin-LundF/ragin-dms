//! The catalog: the aggregate root of a library, and merging two copies of it.

mod catalog_merge_conflict;
mod catalog_merger;
mod catalog_rejection;
mod catalog_update;
pub mod model;

pub use catalog_merge_conflict::CatalogMergeConflict;
pub use catalog_merger::CatalogMerger;
pub use catalog_rejection::CatalogRejection;
pub use catalog_update::CatalogUpdate;
pub use model::{Catalog, CatalogMergeResult, CatalogRevision, LibraryId};
