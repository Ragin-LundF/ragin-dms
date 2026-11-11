//! Tests of `dms_application::assistant`, ported from `dms-application/src/test/kotlin/dms/application/assistant`.

mod ask_about_document;
mod suggest_tags;
mod summarize_document;
mod tag_all_untagged;

use dms_application::catalog::CatalogEdit;
use dms_domain::document::DocumentId;
use dms_domain::taxonomy::TagName;
use indexmap::IndexSet;

use crate::support::TestLibrary;
use crate::support::test_data::done;

/// Kotlin's `library.editCatalog(CatalogEdit.AddTags(..))`.
fn add_tags(library: &TestLibrary, document_id: &DocumentId, tags: &[TagName]) {
    let edit =
        CatalogEdit::AddTags { document_id: document_id.clone(), tags: tags.iter().cloned().collect::<IndexSet<_>>() };
    done(library.edit_catalog().run(&edit));
}
