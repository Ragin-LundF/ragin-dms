//! Catalog files as the released versions write them, and as Jackson writes them back.

use dms_application::port::storage::CatalogRepository;
use dms_domain::document::DocumentId;

use super::Library;
use crate::support::{Applied, fixture};

const V1_0: &str = "catalog/format-1-v1.0.yaml";
const V1_1: &str = "catalog/format-1-v1.1.yaml";

#[test]
fn a_file_written_by_1_0_loads() {
    let catalog = Library::new().load_text(&fixture(V1_0));

    assert_eq!(catalog.documents().len(), 2);
    assert_eq!(catalog.categories().values().map(|category| category.name.as_str()).collect::<Vec<_>>(), ["Invoices"]);
    assert_eq!(catalog.revision().value(), 3);
}

#[test]
fn a_file_written_by_1_1_loads_with_every_1_1_section() {
    let catalog = Library::new().load_text(&fixture(V1_1));

    let settings = catalog.settings();
    assert_eq!(settings.field_definitions.len(), 6);
    assert_eq!(settings.saved_searches.len(), 1);
    assert_eq!(settings.rules.len(), 1);
    assert_eq!(settings.correspondents.len(), 1);
    assert!(settings.layout.inbox.enabled);
    assert_eq!(catalog.links().len(), 1);
    assert!(catalog.documents()[&DocumentId::new("doc-b").expect("id")].retention.is_some());
}

#[test]
fn each_fixture_round_trips_unchanged_to_the_byte() {
    for name in [V1_0, V1_1] {
        let library = Library::new();
        let original = fixture(name);
        let catalog = library.load_text(&original);

        library.save(&catalog);

        assert_eq!(library.text(), original, "{name}");
        assert_eq!(library.found(), catalog, "{name}");
    }
}

#[test]
fn a_1_0_file_edited_by_1_1_keeps_every_unchanged_line() {
    let library = Library::new();
    let original = fixture(V1_0);
    let catalog = library.load_text(&original);
    let edited = catalog.set_note(&DocumentId::new("doc-b").expect("id"), Some("Added by 1.1")).applied();

    library.repository.save(&library.folder, &edited, catalog.revision()).expect("saved");

    let written = library.text();
    let written: Vec<&str> = written.lines().collect();
    let kept: Vec<&str> = original.lines().filter(|line| !line.starts_with("revision:")).collect();
    assert_eq!(written.iter().filter(|line| kept.contains(line)).copied().collect::<Vec<_>>(), kept);
    assert!(written.contains(&"revision: 4"));
    assert!(written.contains(&"  note: Added by 1.1"));
}

/// The Jackson golden files: what the Kotlin version wrote after loading and saving the same input.
#[test]
fn a_saved_catalog_has_the_bytes_jackson_wrote() {
    for (input, golden) in [
        ("catalog/odd-catalog.yaml", "jackson/saved-odd-catalog.yaml"),
        ("catalog/future-fields.yaml", "jackson/saved-future-fields.yaml"),
        (V1_1, "jackson/saved-format-1-v1.1.yaml"),
    ] {
        let library = Library::new();
        let catalog = library.load_text(&fixture(input));

        library.save(&catalog);

        assert_eq!(library.text(), fixture(golden), "{input}");
    }
}

#[test]
fn a_file_jackson_wrote_is_saved_again_unchanged() {
    for golden in ["jackson/saved-odd-catalog.yaml", "jackson/saved-future-fields.yaml"] {
        let library = Library::new();
        let catalog = library.load_text(&fixture(golden));

        library.save(&catalog);

        assert_eq!(library.text(), fixture(golden), "{golden}");
    }
}
