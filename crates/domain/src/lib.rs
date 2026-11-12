//! Domain types and pure rules of ragin-dms, ported from the Kotlin module `dms-domain`. No I/O, no
//! serialization. Each module mirrors one Kotlin package.

pub mod bill;
pub mod cases;
pub mod catalog;
pub mod change;
pub mod classification;
pub mod contract;
mod dates;
pub mod deadline;
pub mod document;
pub mod emergency;
mod error;
pub mod expectation;
pub mod field;
pub mod health;
pub mod library;
pub mod link;
mod macros;
pub mod money;
pub mod outline;
pub mod query;
pub mod rebuild;
pub mod restructure;
pub mod retention;
pub mod rule;
pub mod settings;
pub mod split;
pub mod task;
pub mod tax;
pub mod taxonomy;
pub mod text;

pub use dates::{iso as iso_date, parse_iso as parse_iso_date};
pub use error::DomainError;
