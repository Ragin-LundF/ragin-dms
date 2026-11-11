//! The one document filter used by the library, smart folders, restructure, export and Ask.

mod field_comparison;
pub mod model;

pub use field_comparison::FieldComparison;
pub use model::{DateRange, DocumentQuery, FieldCondition, QueryContext, SavedSearch, SavedSearchId, TagFilter};
