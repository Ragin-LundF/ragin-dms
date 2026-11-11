use dms_domain::classification::ClassificationSuggestion;

use crate::test_data::document_id;

fn suggestion() -> ClassificationSuggestion {
    ClassificationSuggestion::empty(document_id("d1"), "local-model")
}

#[test]
fn a_suggested_file_name_must_satisfy_the_file_name_policy() {
    let invalid = ClassificationSuggestion { file_name: Some("a:b.pdf".to_owned()), ..suggestion() };
    let valid = ClassificationSuggestion { file_name: Some("a b.pdf".to_owned()), ..suggestion() };

    assert!(invalid.validate().is_err());
    assert!(valid.validate().is_ok());
}

#[test]
fn confidence_must_be_between_zero_and_one() {
    assert!(ClassificationSuggestion { confidence: Some(1.5), ..suggestion() }.validate().is_err());
    assert!(ClassificationSuggestion { confidence: Some(1.0), ..suggestion() }.validate().is_ok());
}
