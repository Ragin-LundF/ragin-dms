//! Manual catalog edits (categories, tags, metadata) and the library-wide settings.

mod change_settings;
mod edit_catalog;
pub mod model;

pub use change_settings::ChangeSettings;
pub use edit_catalog::EditCatalog;
pub use model::CatalogEdit;
