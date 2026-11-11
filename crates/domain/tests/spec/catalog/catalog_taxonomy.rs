use dms_domain::catalog::{Catalog, CatalogRejection, CatalogUpdate};
use dms_domain::document::Document;
use dms_domain::taxonomy::{Category, CategoryId};
use indexmap::indexset;

use crate::test_data::{self, Applied, category_id, tag};

fn category(id: &str, name: &str, parent: Option<&CategoryId>) -> Category {
    Category::new(category_id(id), name, parent.cloned(), None).expect("valid category")
}

fn finance() -> Category {
    category("finance", "Finance", None)
}

fn invoices() -> Category {
    category("invoices", "Invoices", Some(&finance().id))
}

fn taxonomy() -> Catalog {
    Catalog::empty(test_data::library_id()).add_category(finance()).applied().add_category(invoices()).applied()
}

#[test]
fn a_category_needs_a_known_parent_and_a_unique_name_among_its_siblings() {
    assert_eq!(
        taxonomy().add_category(category("x", "X", Some(&category_id("nope")))),
        Ok(CatalogUpdate::Rejected(CatalogRejection::CategoryNotFound))
    );
    assert_eq!(
        taxonomy().add_category(category("x", " invoices ", Some(&finance().id))),
        Ok(CatalogUpdate::Rejected(CatalogRejection::CategoryNameTaken))
    );
}

#[test]
fn the_same_name_is_allowed_below_different_parents() {
    let catalog = taxonomy().add_category(category("other", "Invoices", None)).applied();

    assert_eq!(catalog.categories().len(), 3);
}

#[test]
fn moving_a_category_below_its_own_descendant_is_rejected() {
    assert_eq!(
        taxonomy().move_category(&finance().id, Some(&invoices().id)),
        CatalogUpdate::Rejected(CatalogRejection::CategoryCycle)
    );
    assert_eq!(
        taxonomy().move_category(&finance().id, Some(&finance().id)),
        CatalogUpdate::Rejected(CatalogRejection::CategoryCycle)
    );
}

#[test]
fn a_category_in_use_or_with_children_cannot_be_removed() {
    let document = Document { category_id: Some(invoices().id), ..test_data::document("d1", "a.pdf") };
    let catalog = taxonomy().add_document(document).applied();

    assert_eq!(catalog.remove_category(&invoices().id), CatalogUpdate::Rejected(CatalogRejection::CategoryInUse));
    assert_eq!(catalog.remove_category(&finance().id), CatalogUpdate::Rejected(CatalogRejection::CategoryHasChildren));
}

#[test]
fn assigning_and_clearing_a_category() {
    let document = test_data::document("d1", "a.pdf");
    let assigned = taxonomy()
        .add_document(document.clone())
        .applied()
        .assign_category(&document.id, Some(invoices().id))
        .applied();

    assert_eq!(assigned.documents()[&document.id].category_id, Some(invoices().id));
    let cleared = assigned.assign_category(&document.id, None).applied();
    assert_eq!(cleared.documents()[&document.id].category_id, None);
}

#[test]
fn renaming_a_tag_into_an_existing_tag_merges_both() {
    let first = Document { tags: indexset! { tag("Tax") }, ..test_data::document("d1", "a.pdf") };
    let second = Document { tags: indexset! { tag("Taxes") }, ..test_data::document("d2", "b.pdf") };

    let merged = test_data::catalog_with(&[first, second]).rename_tag(&tag("Taxes"), tag("tax")).applied();

    assert_eq!(merged.tags().iter().map(dms_domain::taxonomy::TagName::value).collect::<Vec<_>>(), vec!["Tax"]);
    assert!(
        merged
            .documents()
            .values()
            .all(|document| { document.tags.len() == 1 && document.tags.iter().all(|tag| tag.value() == "Tax") })
    );
}

#[test]
fn renaming_a_tag_to_another_casing_changes_its_spelling() {
    let document = Document { tags: indexset! { tag("tax") }, ..test_data::document("d1", "a.pdf") };

    let renamed =
        test_data::catalog_with(std::slice::from_ref(&document)).rename_tag(&tag("tax"), tag("Tax")).applied();

    assert_eq!(renamed.tags().iter().map(dms_domain::taxonomy::TagName::value).collect::<Vec<_>>(), vec!["Tax"]);
    assert_eq!(
        renamed.documents()[&document.id].tags.iter().map(dms_domain::taxonomy::TagName::value).collect::<Vec<_>>(),
        vec!["Tax"]
    );
}

#[test]
fn removing_a_tag_removes_it_from_every_document() {
    let draft = tag("Draft");
    let document = Document { tags: indexset! { draft.clone() }, ..test_data::document("d1", "a.pdf") };

    let catalog = test_data::catalog_with(std::slice::from_ref(&document)).remove_tag(&draft).applied();

    assert!(catalog.tags().is_empty());
    assert!(catalog.documents()[&document.id].tags.is_empty());
    assert_eq!(catalog.remove_tag(&draft), CatalogUpdate::Rejected(CatalogRejection::TagNotFound));
}
