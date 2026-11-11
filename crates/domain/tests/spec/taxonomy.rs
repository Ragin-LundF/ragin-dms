use std::collections::HashSet;
use std::hash::{BuildHasher, RandomState};

use dms_domain::taxonomy::TagName;

use crate::test_data::tag;

#[test]
fn whitespace_is_trimmed_and_collapsed() {
    assert_eq!(tag("  Home \t  Office ").value(), "Home Office");
}

#[test]
fn tags_are_equal_regardless_of_case_but_keep_their_spelling() {
    let upper = tag("Tax");
    let lower = tag("tax");
    let hasher = RandomState::new();

    assert_eq!(upper, lower);
    assert_eq!(hasher.hash_one(&upper), hasher.hash_one(&lower));
    assert_eq!(upper.value(), "Tax");
    assert_eq!([upper.clone(), lower].into_iter().collect::<HashSet<_>>().len(), 1);
    assert_ne!(upper, tag("Taxes"));
}

#[test]
fn blank_and_overlong_tags_are_rejected() {
    assert!(TagName::of("   ").is_err());
    assert!(TagName::of(&"x".repeat(TagName::MAX_LENGTH + 1)).is_err());
}

#[test]
fn parse_gives_the_same_tag_as_of_and_nothing_for_a_blank_or_too_long_one() {
    let parsed = TagName::parse("  Tax   2024 ");

    assert_eq!(parsed, Some(tag("  Tax   2024 ")));
    assert_eq!(parsed.as_ref().map(TagName::value), Some("Tax 2024"));
    assert_eq!(TagName::parse("   "), None);
    assert_eq!(TagName::parse(&"x".repeat(TagName::MAX_LENGTH + 1)), None);
}
