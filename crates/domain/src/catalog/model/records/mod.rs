//! The records a library keeps beside its documents: contracts, cases and tax years, and the catalog functions
//! that change them. A feature that needs more catalog functions adds a `catalog_<feature>.rs` here.

mod catalog_bills;
mod catalog_record_edits;
mod catalog_records;

pub use catalog_records::CatalogRecords;
