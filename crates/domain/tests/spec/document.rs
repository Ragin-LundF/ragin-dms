use dms_domain::document::{ContentHash, DocumentKind, FileNameDates};

use crate::test_data::{date, path};

#[test]
fn upper_case_hex_is_normalized_to_lower_case() {
    let hash = ContentHash::from_hex(&"AB".repeat(32)).expect("valid hex");

    assert_eq!(hash.value(), "ab".repeat(32));
}

#[test]
fn anything_but_64_hex_characters_is_rejected() {
    for value in [String::new(), "abc".to_owned(), "g".repeat(64), "a".repeat(65)] {
        assert!(ContentHash::new(value.clone()).is_err(), "{value}");
    }
}

#[test]
fn kind_is_decided_by_the_extension_regardless_of_case() {
    let expectations = [
        ("a/scan.PDF", DocumentKind::Pdf),
        ("photo.jpeg", DocumentKind::Image),
        ("photo.png", DocumentKind::Image),
        ("letter.docx", DocumentKind::Office),
        ("slides.pptx", DocumentKind::Office),
        ("sheet.xlsx", DocumentKind::Office),
        ("audio.mp3", DocumentKind::Other),
        ("no-extension", DocumentKind::Other),
    ];

    for (raw, kind) in expectations {
        assert_eq!(DocumentKind::of(&path(raw)), kind, "{raw}");
    }
}

#[test]
fn other_files_are_not_readable_by_the_ai() {
    assert!(!DocumentKind::Other.readable_by_ai());
}

#[test]
fn common_date_formats_are_recognized() {
    let expectations = [
        "2024-03-15 Sample bill.pdf",
        "scan_2024_03_15.pdf",
        "2024.03.15 note.pdf",
        "IMG_20240315_101500.jpg",
        "Sample 15.03.2024.pdf",
        "15-03-2024 sample.pdf",
    ];
    for name in expectations {
        assert_eq!(FileNameDates::date_in(name), Some(date(2024, 3, 15)), "{name}");
    }
    assert_eq!(FileNameDates::date_in("2024-03 Sample.pdf"), Some(date(2024, 3, 1)));
}

#[test]
fn the_leftmost_date_wins() {
    assert_eq!(FileNameDates::date_in("2023-12-01 reply to 15.11.2023.pdf"), Some(date(2023, 12, 1)));
}

#[test]
fn numbers_that_are_no_date_are_ignored() {
    for name in ["Sample 12345678.pdf", "2024-13-40 Sample.pdf", "Order 0815.pdf", "1234-56.pdf", "Sample.pdf"] {
        assert_eq!(FileNameDates::date_in(name), None, "{name}");
    }
}
