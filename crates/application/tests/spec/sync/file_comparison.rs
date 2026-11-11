use dms_application::sync::FileComparison;
use dms_domain::catalog::Catalog;
use dms_domain::change::FileFacts;
use dms_domain::library::RelativePath;

use crate::support::TestLibrary;
use crate::support::fake::storage::FakeFolders;
use crate::support::test_data::{Applied, library_id, path};

const COMPOSED: &str = "Bel\u{e4}ge/\u{fc}bersicht.pdf";
/// `COMPOSED` in Unicode NFD, as macOS file systems may report it.
const DECOMPOSED: &str = "Bela\u{308}ge/u\u{308}bersicht.pdf";

fn notice() -> RelativePath {
    path("letters/notice.pdf")
}

fn catalog_with(raw_path: &str, content: &str) -> Catalog {
    Catalog::empty(library_id("library")).add_document(TestLibrary::document("doc-1", raw_path, content)).applied()
}

fn catalog() -> Catalog {
    catalog_with("letters/notice.pdf", "catalogued")
}

fn catalogued() -> FileFacts {
    FakeFolders::facts_of("catalogued")
}

fn unhashed(facts: &FileFacts) -> FileFacts {
    FileFacts { size: facts.size, content_hash: None }
}

#[test]
fn the_hash_the_scanner_computed_wins_over_the_catalogued_one() {
    let facts = FakeFolders::facts_of("changed");
    let catalog = catalog();

    let hash = FileComparison::effective_hash(&facts, &notice(), &catalog);

    assert_eq!(hash, Some(&FakeFolders::hash_of("changed")));
}

#[test]
fn a_file_the_scanner_did_not_hash_has_the_catalogued_content() {
    let facts = unhashed(&catalogued());
    let catalog = catalog();

    let hash = FileComparison::effective_hash(&facts, &notice(), &catalog);

    assert_eq!(hash, Some(&FakeFolders::hash_of("catalogued")));
}

#[test]
fn an_unhashed_file_the_catalog_does_not_know_has_no_hash() {
    let facts = unhashed(&catalogued());
    let catalog = catalog();

    let hash = FileComparison::effective_hash(&facts, &path("letters/other.pdf"), &catalog);

    assert_eq!(hash, None);
}

#[test]
fn a_decomposed_path_finds_the_catalogued_hash_of_its_composed_form() {
    let composed_catalog = catalog_with(COMPOSED, "catalogued");
    let facts = unhashed(&catalogued());

    let hash = FileComparison::effective_hash(&facts, &path(DECOMPOSED), &composed_catalog);

    assert_ne!(COMPOSED, DECOMPOSED);
    assert_eq!(hash, Some(&FakeFolders::hash_of("catalogued")));
}

#[test]
fn two_copies_with_the_same_hash_do_not_differ() {
    let differs =
        FileComparison::differs(&FakeFolders::facts_of("same"), &FakeFolders::facts_of("same"), &notice(), &catalog());

    assert!(!differs);
}

#[test]
fn copies_of_the_same_size_with_different_content_differ() {
    let differs =
        FileComparison::differs(&FakeFolders::facts_of("aaaa"), &FakeFolders::facts_of("bbbb"), &notice(), &catalog());

    assert!(differs);
}

#[test]
fn equal_hashes_decide_even_when_the_reported_sizes_disagree() {
    let first = FakeFolders::facts_of("same");
    let second = FileFacts { size: first.size + 1, ..first.clone() };

    let differs = FileComparison::differs(&first, &second, &notice(), &catalog());

    assert!(!differs);
}

#[test]
fn two_unhashed_copies_of_a_catalogued_file_both_have_the_catalogued_content() {
    let differs = FileComparison::differs(&unhashed(&catalogued()), &unhashed(&catalogued()), &notice(), &catalog());

    assert!(!differs);
}

#[test]
fn an_unhashed_copy_matches_a_hashed_copy_with_the_catalogued_content() {
    let differs = FileComparison::differs(&unhashed(&catalogued()), &catalogued(), &notice(), &catalog());

    assert!(!differs);
}

#[test]
fn a_changed_copy_of_the_same_size_differs_from_an_unhashed_catalogued_copy() {
    let changed = FakeFolders::facts_of("CATALOGUED");

    let differs = FileComparison::differs(&unhashed(&catalogued()), &changed, &notice(), &catalog());

    assert_eq!(changed.size, catalogued().size);
    assert!(differs);
}

#[test]
fn a_content_conflict_is_found_under_a_decomposed_path_although_the_sizes_match() {
    let composed_catalog = catalog_with(COMPOSED, "catalogued");

    let differs = FileComparison::differs(
        &unhashed(&catalogued()),
        &FakeFolders::facts_of("CATALOGUED"),
        &path(DECOMPOSED),
        &composed_catalog,
    );

    assert!(differs);
}

#[test]
fn without_a_hash_on_one_side_an_unknown_file_falls_back_to_its_size() {
    let other = path("letters/other.pdf");
    let catalog = catalog();

    let same_size = FileComparison::differs(
        &unhashed(&FakeFolders::facts_of("aaaa")),
        &FakeFolders::facts_of("bbbb"),
        &other,
        &catalog,
    );
    let other_size = FileComparison::differs(
        &unhashed(&FakeFolders::facts_of("aaaa")),
        &FakeFolders::facts_of("bbbbb"),
        &other,
        &catalog,
    );

    assert!(!same_size);
    assert!(other_size);
}

#[test]
fn without_any_hash_an_unknown_file_falls_back_to_its_size() {
    let other = path("letters/other.pdf");
    let catalog = catalog();
    let facts = |size| FileFacts { size, content_hash: None };

    let same_size = FileComparison::differs(&facts(10), &facts(10), &other, &catalog);
    let other_size = FileComparison::differs(&facts(10), &facts(11), &other, &catalog);

    assert!(!same_size);
    assert!(other_size);
}
