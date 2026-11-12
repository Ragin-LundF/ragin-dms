//! The tax year folder: which documents belong to which tax year and area, and the checklist per year.

pub mod model;
mod tax_area;
mod tax_checklist_builder;
mod tax_item_state;
mod tax_sums;

pub use model::{TaxAssignment, TaxChecklistItem, TaxFilter, TaxItemId, TaxYear};
pub use tax_area::TaxArea;
pub use tax_checklist_builder::TaxChecklistBuilder;
pub use tax_item_state::TaxItemState;
pub use tax_sums::TaxSums;
