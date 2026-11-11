//! Domain types and pure rules of ragin-dms, ported from the Kotlin module `dms-domain`. No I/O, no
//! serialization. Each module mirrors one Kotlin package.

pub mod catalog;
pub mod change;
pub mod classification;
mod dates;
pub mod deadline;
pub mod document;
mod error;
pub mod field;
pub mod library;
pub mod link;
mod macros;
pub mod outline;
pub mod query;
pub mod rebuild;
pub mod restructure;
pub mod retention;
pub mod rule;
pub mod settings;
pub mod task;
pub mod taxonomy;
pub mod text;

pub use dates::{iso as iso_date, parse_iso as parse_iso_date};
pub use error::DomainError;
