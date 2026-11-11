use dms_domain::library::{FileNamePolicy, FileNameViolation};

#[test]
fn ordinary_names_with_spaces_umlauts_and_dates_are_valid() {
    for name in ["2024-01-15 Example Invoice.pdf", "\u{dc}bersicht M\u{e4}rz.docx", "notes", "a.b.c"] {
        assert_eq!(FileNamePolicy::check(name), None, "{name}");
    }
}

#[test]
fn each_rule_reports_its_own_violation() {
    let too_long = "x".repeat(FileNamePolicy::MAX_LENGTH + 1);
    let expectations = [
        ("", FileNameViolation::Empty),
        (too_long.as_str(), FileNameViolation::TooLong),
        ("tab\there", FileNameViolation::ControlCharacter),
        ("a:b.pdf", FileNameViolation::ForbiddenCharacter),
        ("a/b.pdf", FileNameViolation::ForbiddenCharacter),
        ("what?.pdf", FileNameViolation::ForbiddenCharacter),
        (".hidden", FileNameViolation::LeadingDotOrSpace),
        (" lead.pdf", FileNameViolation::LeadingDotOrSpace),
        ("trail.", FileNameViolation::TrailingDotOrSpace),
        ("trail ", FileNameViolation::TrailingDotOrSpace),
        ("CON", FileNameViolation::ReservedName),
        ("nul.txt", FileNameViolation::ReservedName),
        ("Com1.pdf", FileNameViolation::ReservedName),
    ];

    for (name, violation) in expectations {
        assert_eq!(FileNamePolicy::check(name), Some(violation), "{name}");
    }
}

#[test]
fn names_that_only_contain_a_reserved_word_are_valid() {
    assert_eq!(FileNamePolicy::check("CONTRACT.pdf"), None);
    assert_eq!(FileNamePolicy::check("COM10.pdf"), None);
}

#[test]
fn names_differing_only_by_case_or_normalization_are_the_same_name() {
    let decomposed = "U\u{308}bersicht.pdf";

    assert!(FileNamePolicy::is_same_name("Report.PDF", "report.pdf"));
    assert!(FileNamePolicy::is_same_name(decomposed, "\u{fc}bersicht.pdf"));
    assert!(!FileNamePolicy::is_same_name("report-1.pdf", "report-2.pdf"));
}

#[test]
fn the_next_free_name_adds_a_counter_before_the_extension() {
    let siblings = ["report.pdf", "Report (2).pdf", "notes"];

    assert_eq!(FileNamePolicy::next_free_name("new.pdf", &siblings), "new.pdf");
    assert_eq!(FileNamePolicy::next_free_name("report.pdf", &siblings), "report (3).pdf");
    assert_eq!(FileNamePolicy::next_free_name("notes", &siblings), "notes (2)");
}

#[test]
fn collision_check_looks_at_all_siblings() {
    let siblings = ["alpha.pdf", "Beta.pdf"];

    assert!(FileNamePolicy::collides_with_any("beta.pdf", siblings));
    assert!(!FileNamePolicy::collides_with_any("gamma.pdf", siblings));
}

#[test]
fn sanitize_turns_any_text_into_a_valid_name() {
    assert_eq!(FileNamePolicy::sanitize("Car: Policies"), "Car- Policies");
    assert_eq!(FileNamePolicy::sanitize(" ..hidden. "), "hidden");
    assert_eq!(FileNamePolicy::sanitize("CON"), "CON_");
    assert_eq!(FileNamePolicy::sanitize("..."), "_");
    assert_eq!(FileNamePolicy::sanitize(&"x".repeat(300)).len(), FileNamePolicy::MAX_LENGTH);
    for name in ["a/b".to_owned(), "tab\there".to_owned(), "\u{dc}".repeat(250)] {
        assert_eq!(FileNamePolicy::check(&FileNamePolicy::sanitize(&name)), None, "{name}");
    }
}
