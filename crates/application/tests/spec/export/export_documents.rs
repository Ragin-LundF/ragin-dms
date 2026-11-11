use dms_application::export::{ExportOptions, ExportOutcome, ExportScope};
use dms_domain::query::DocumentQuery;

use crate::support::TestLibrary;

#[test]
fn flattening_gives_clashing_names_a_suffix_ignoring_case() {
    let library = TestLibrary::new();
    library.open();
    let first = library.add("2024/scan.pdf", "a");
    let second = library.add("2025/SCAN.pdf", "b");

    let outcome = library.export_documents().run(
        &ExportScope::Documents(vec![first.id, second.id]),
        "/outside/export",
        "Bundle",
        ExportOptions { flatten: true, ..ExportOptions::default() },
    );

    assert_eq!(outcome, ExportOutcome::Exported { count: 2, location: "/outside/export/Bundle".to_owned() });
    let written = library.exports.written.get();
    assert_eq!(written.len(), 1);
    let (bundle, target) = &written[0];
    let targets: Vec<&str> = bundle.entries.iter().map(|entry| entry.target.as_str()).collect();
    assert_eq!(targets, vec!["scan.pdf", "SCAN (2).pdf"]);
    assert!(!target.zip);
}

#[test]
fn an_export_never_writes_into_a_library_folder_and_an_empty_scope_writes_nothing() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("a.pdf", "a");
    let export = library.export_documents();

    let inside = export.run(
        &ExportScope::Documents(vec![document.id]),
        &format!("{}/exports", library.sync_a.location.value()),
        "x",
        ExportOptions::default(),
    );
    let empty = export.run(
        &ExportScope::Query(DocumentQuery {
            text: Some("nothing matches this".to_owned()),
            ..DocumentQuery::default()
        }),
        "/outside",
        "x",
        ExportOptions::default(),
    );

    assert_eq!(inside, ExportOutcome::InsideLibrary);
    assert_eq!(empty, ExportOutcome::Empty);
    assert_eq!(library.exports.written.lock().len(), 0);
}
