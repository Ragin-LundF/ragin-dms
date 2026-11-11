//! What the app derives per document and keeps in hidden folders of every library folder: the stored previews,
//! and the layout of those folders (`<xx>/` subfolders named after the first two characters of the document id).

pub mod model;
mod preview_store;
mod sidecar_layout;

pub use model::{SidecarFile, SidecarKind, SidecarMove};
pub use preview_store::PreviewStore;
pub use sidecar_layout::SidecarLayout;
