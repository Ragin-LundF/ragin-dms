use dms_domain::library::TemporaryNames;

use crate::test_data::path;

#[test]
fn a_sibling_lies_in_the_same_folder_starts_with_the_prefix_and_keeps_the_extension() {
    let sibling = TemporaryNames::sibling_of(&path("letters/scan.pdf"), "42").expect("valid id");

    assert_eq!(sibling.value(), "letters/.dms-tmp-42.pdf");
    assert!(TemporaryNames::is_temporary(sibling.file_name()));
}

#[test]
fn a_sibling_of_a_file_at_the_top_level_without_an_extension_has_none() {
    let sibling = TemporaryNames::sibling_of(&path("notes"), "7").expect("valid id");

    assert_eq!(sibling.value(), ".dms-tmp-7");
}

#[test]
fn an_ordinary_name_is_not_temporary() {
    assert!(!TemporaryNames::is_temporary("dms-tmp-report.pdf"));
}
