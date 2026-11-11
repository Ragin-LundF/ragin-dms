use dms_domain::catalog::{Catalog, CatalogRejection, CatalogUpdate};
use dms_domain::document::{Document, DocumentId};
use dms_domain::link::DocumentLink;

use crate::test_data::{self, Applied};

struct Fixture {
    invoice: Document,
    reminder: Document,
    receipt: Document,
    catalog: Catalog,
}

fn fixture() -> Fixture {
    let invoice = test_data::document("invoice", "invoice.pdf");
    let reminder = test_data::document("reminder", "reminder.pdf");
    let receipt = test_data::document("receipt", "receipt.pdf");
    let catalog = test_data::catalog_with(&[invoice.clone(), reminder.clone(), receipt.clone()]);
    Fixture { invoice, reminder, receipt, catalog }
}

fn link(first: &DocumentId, second: &DocumentId, label: Option<&str>) -> DocumentLink {
    DocumentLink::new(first.clone(), second.clone(), label).expect("valid link")
}

#[test]
fn a_link_is_the_same_from_both_ends() {
    let f = fixture();

    let linked = f.catalog.link(link(&f.reminder.id, &f.invoice.id, None)).applied();

    assert_eq!(linked.links_of(&f.invoice.id).len(), 1);
    assert_eq!(linked.links_of(&f.reminder.id)[0].other_than(&f.reminder.id), &f.invoice.id);
    assert_eq!(link(&f.invoice.id, &f.reminder.id, None), link(&f.reminder.id, &f.invoice.id, None));
}

#[test]
fn linking_the_same_pair_again_replaces_the_label() {
    let f = fixture();

    let linked = f
        .catalog
        .link(link(&f.invoice.id, &f.reminder.id, Some("first")))
        .applied()
        .link(link(&f.reminder.id, &f.invoice.id, Some("reminder")))
        .applied();

    assert_eq!(linked.links().len(), 1);
    assert_eq!(linked.links()[0].label(), Some("reminder"));
}

#[test]
fn a_document_cannot_be_linked_to_itself() {
    let f = fixture();

    assert!(DocumentLink::new(f.invoice.id.clone(), f.invoice.id, None).is_err());
}

#[test]
fn links_to_unknown_documents_are_refused() {
    let f = fixture();
    let ghost = test_data::document("ghost", "ghost.pdf");

    let update = f.catalog.link(link(&f.invoice.id, &ghost.id, None));

    assert_eq!(update, CatalogUpdate::Rejected(CatalogRejection::DocumentNotFound));
}

#[test]
fn unlink_removes_the_link_from_either_end() {
    let f = fixture();
    let linked = f.catalog.link(link(&f.invoice.id, &f.reminder.id, None)).applied();

    let unlinked = linked.unlink(&f.reminder.id, &f.invoice.id).applied();

    assert!(unlinked.links().is_empty());
    assert_eq!(unlinked.unlink(&f.invoice.id, &f.reminder.id), CatalogUpdate::Rejected(CatalogRejection::LinkNotFound));
}

#[test]
fn removing_a_document_removes_its_links() {
    let f = fixture();
    let linked = f
        .catalog
        .link(link(&f.invoice.id, &f.reminder.id, None))
        .applied()
        .link(link(&f.reminder.id, &f.receipt.id, None))
        .applied();

    let removed = linked.remove_document(&f.reminder.id).applied();

    assert!(removed.links().is_empty());
}
