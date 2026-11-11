//! The per-document files in the hidden folders of a library folder: where they live (`sidecar_paths`), the stored
//! previews, and moving the flat 1.x layout into subfolders.

mod file_preview_store;
mod file_sidecar_layout;
pub mod sidecar_paths;

pub use file_preview_store::FilePreviewStore;
pub use file_sidecar_layout::FileSidecarLayout;
