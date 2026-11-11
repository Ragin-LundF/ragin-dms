//! Tests of `dms_application::inbox`, ported from `dms-application/src/test/kotlin/dms/application/inbox`.

mod inbox_classification;
mod inbox_flow;

use dms_application::library::ScanResult;

use crate::support::TestLibrary;

/// What a scan of every library folder finds (Kotlin's `TestLibrary.scanned()!!`).
#[track_caller]
fn scan_now(library: &TestLibrary) -> ScanResult {
    library.scanned(false).expect("a library is open")
}
