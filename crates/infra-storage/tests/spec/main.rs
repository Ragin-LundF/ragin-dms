//! The tests of the Kotlin module `dms-storage`, one module per Kotlin test class, plus the format tests against
//! the Jackson golden files in `tests/fixtures/jackson`. Everything runs in temporary directories with a recording
//! trash, never against the system trash or a real library.

mod catalog;
mod export;
mod fs;
mod index;
mod plan;
mod scan;
mod sidecar;
mod state;
mod support;
mod text;
mod watch;
