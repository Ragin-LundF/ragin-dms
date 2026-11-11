//! The layout of the hidden per-document folders of every library folder: moving the flat 1.x layout into
//! subfolders.

mod migrate_sidecar_layout;
pub mod model;

pub use migrate_sidecar_layout::MigrateSidecarLayout;
pub use model::{
    FolderMigration, FolderMigrationResult, KeptSidecarFile, SidecarMigrationOutcome, SidecarMigrationProgress,
};
