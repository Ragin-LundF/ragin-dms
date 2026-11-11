//! Property-style round trip: catalogs made of awkward text (YAML keywords, numbers, indicators, umlauts, emoji,
//! line breaks, long lines) load back equal and are written to the same bytes again.

use dms_domain::catalog::{Catalog, LibraryId};
use dms_domain::document::DocumentMetadata;
use dms_domain::field::{FieldDefinition, FieldKey, FieldType, FieldValue};
use dms_domain::settings::CatalogSettings;
use dms_domain::taxonomy::{Category, CategoryId, TagName};
use indexmap::{IndexMap, IndexSet};

use super::{Library, document};
use crate::support::Applied;

const FRAGMENTS: [&str; 44] = [
    "yes",
    "No",
    "on",
    "OFF",
    "~",
    "null",
    "true",
    "2024",
    "0123",
    "1e3",
    "-5",
    ".5",
    "+1",
    "#tag",
    "a #b",
    "a: b",
    "a:",
    "[x]",
    "{y}",
    "a,b",
    "&a",
    "*b",
    "!c",
    "|d",
    ">e",
    "'q'",
    "\"dq\"",
    "%p",
    "@a",
    "`t",
    "---",
    "...",
    "- dash",
    "? q",
    "Straße",
    "Grüße äöü",
    "😀 emoji",
    "Ａ",
    "tab\tin",
    "back\\slash",
    "trailing ",
    "two\nlines",
    "end\n",
    "a \nb",
];

/// A small deterministic generator, so a failure can be reproduced.
struct Random(u64);

impl Random {
    fn next(&mut self, bound: usize) -> usize {
        self.0 = self.0.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        usize::try_from(self.0 >> 33).expect("fits") % bound
    }

    fn text(&mut self) -> String {
        let parts = 1 + self.next(4);
        let mut text: String = (0..parts).map(|_| FRAGMENTS[self.next(FRAGMENTS.len())]).collect::<Vec<_>>().join(" ");
        if self.next(5) == 0 {
            text = format!("{text} {}", "long words make the emitter wrap the line ".repeat(1 + self.next(3)));
        }
        text
    }
}

fn odd_catalog(random: &mut Random) -> Catalog {
    let note_key = FieldKey::new("remark").expect("key");
    let settings = CatalogSettings {
        field_definitions: vec![
            FieldDefinition::new(note_key.clone(), random.text().trim(), FieldType::Text, None).expect("field"),
        ],
        ..CatalogSettings::default()
    };
    let category =
        Category::new(CategoryId::new("cat").expect("id"), random.text().trim(), None, None).expect("category");
    let mut catalog = Catalog::empty(LibraryId::new("library").expect("id"))
        .change_settings(settings)
        .applied()
        .add_category(category.clone())
        .applied();
    for index in 0..8 {
        let mut document =
            document(&format!("doc-{index}"), &format!("{index} {}.pdf", random.text().replace('/', "-")), "x");
        document.metadata = DocumentMetadata::new(Some(random.text()), None, Some(random.text())).expect("metadata");
        document.tags = (0..3).filter_map(|_| TagName::parse(&random.text())).collect::<IndexSet<_>>();
        document.note = Some(random.text());
        document.category_id = Some(category.id.clone());
        document.fields = IndexMap::from([(note_key.clone(), FieldValue::text(random.text()).expect("text"))]);
        let mut tags = catalog.tags().clone();
        tags.extend(document.tags.iter().cloned());
        catalog = Catalog::new(
            catalog.library_id().clone(),
            catalog.revision(),
            catalog.documents().clone(),
            catalog.categories().clone(),
            tags,
            catalog.links().clone(),
            catalog.settings().clone(),
        )
        .expect("catalog")
        .add_document(document)
        .applied();
    }
    catalog
}

#[test]
fn catalogs_with_odd_text_load_back_equal_and_save_to_the_same_bytes() {
    let mut random = Random(20_261_110);
    for round in 0..60 {
        let library = Library::new();
        let catalog = odd_catalog(&mut random);

        library.save(&catalog);
        let first = library.text();
        let loaded = library.found();
        library.save(&loaded);

        assert_eq!(loaded, catalog, "round {round}:\n{first}");
        assert_eq!(library.text(), first, "round {round}");
    }
}
