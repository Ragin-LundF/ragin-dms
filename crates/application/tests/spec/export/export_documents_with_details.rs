//! The `ExportDocumentsTest` case that edits document details first.

use dms_application::catalog::CatalogEdit;
use dms_application::document::DetailEdit;
use dms_application::export::{ExportOptions, ExportScope};
use dms_domain::document::DocumentMetadata;
use dms_domain::field::{FieldPresets, FieldValue};
use dms_domain::query::{DocumentQuery, TagFilter};
use dms_domain::settings::CatalogSettings;
use indexmap::{IndexMap, IndexSet};

use crate::support::TestLibrary;
use crate::support::test_data::{date, path, tag};

#[test]
fn renaming_to_the_title_keeps_the_extension_and_the_index_carries_every_value() {
    let library = TestLibrary::new();
    library.open();
    library.change_settings().run("fields", |_| CatalogSettings {
        field_definitions: vec![FieldPresets::due_date()],
        ..CatalogSettings::default()
    });
    let document = library.add("letters/scan.pdf", "a");
    library.edit_catalog().run(&CatalogEdit::UpdateMetadata {
        document_id: document.id.clone(),
        metadata: DocumentMetadata {
            title: Some("Power: bill".to_owned()),
            document_date: Some(date(2024, 2, 1)),
            correspondent: Some("Sample Energy".to_owned()),
        },
    });
    library
        .edit_catalog()
        .run(&CatalogEdit::AddTags { document_id: document.id.clone(), tags: IndexSet::from([tag("Tax")]) });
    library.edit_document_details().run(&DetailEdit::SetFields {
        document_id: document.id.clone(),
        fields: IndexMap::from([(FieldPresets::due_date().key, FieldValue::Date(date(2024, 3, 1)))]),
    });
    library
        .edit_document_details()
        .run(&DetailEdit::SetNote { document_id: document.id.clone(), note: Some("paid".to_owned()) });

    library.export_documents().run(
        &ExportScope::Query(DocumentQuery {
            tags: TagFilter { any: IndexSet::from([tag("tax")]), ..TagFilter::default() },
            ..DocumentQuery::default()
        }),
        "/outside",
        "Tax 2024",
        ExportOptions { rename_to_title: true, zip: true, ..ExportOptions::default() },
    );

    let written = library.exports.written.get();
    assert_eq!(written.len(), 1);
    let (bundle, target) = &written[0];
    let targets: Vec<&str> = bundle.entries.iter().map(|entry| entry.target.as_str()).collect();
    assert_eq!(targets, vec!["letters/Power- bill.pdf"]);
    assert_eq!(bundle.entries[0].source, path("letters/scan.pdf"));
    let rows = bundle.index.clone().unwrap_or_default();
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(
        vec![
            row.date.as_str(),
            row.correspondent.as_str(),
            row.tags.as_str(),
            row.fields[0].as_str(),
            row.note.as_str()
        ],
        vec!["2024-02-01", "Sample Energy", "Tax", "2024-03-01", "paid"]
    );
    assert_eq!(bundle.field_labels, vec!["Due date"]);
    assert!(target.zip);
}
