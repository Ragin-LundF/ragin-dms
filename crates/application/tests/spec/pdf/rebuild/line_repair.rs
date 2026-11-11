use dms_application::pdf::rebuild::LineRepair;

#[test]
fn nul_and_the_replacement_character_count_as_lost() {
    assert!(LineRepair::has_lost_characters("Gr\0\0e"));
    assert!(LineRepair::has_lost_characters("Gr\u{FFFD}e"));
    assert!(!LineRepair::has_lost_characters("Grüße"));
}

#[test]
fn lost_runs_at_the_start_in_the_middle_and_at_the_end_are_filled() {
    assert_eq!(LineRepair::repaired("\0bung", "Übung").as_deref(), Some("Übung"));
    assert_eq!(LineRepair::repaired("K\0ln", "Köln").as_deref(), Some("Köln"));
    assert_eq!(LineRepair::repaired("Geb\0hr", "Gebühr").as_deref(), Some("Gebühr"));
    assert_eq!(LineRepair::repaired("Ma\0", "Maß").as_deref(), Some("Maß"));
}

#[test]
fn several_runs_are_filled_independently() {
    assert_eq!(LineRepair::repaired("Gr\0\0e aus K\0ln", "Grüße aus Köln").as_deref(), Some("Grüße aus Köln"));
}

#[test]
fn a_run_of_n_lost_characters_may_become_fewer_but_not_more_and_never_none() {
    assert_eq!(LineRepair::repaired("Gr\0\0te", "Gräte").as_deref(), Some("Gräte"));
    assert_eq!(LineRepair::repaired("Gr\0te", "Graete"), None);
    assert_eq!(LineRepair::repaired("Gr\0te", "Grte"), None);
}

#[test]
fn spacing_of_the_reread_does_not_matter_but_a_lost_character_never_becomes_a_space() {
    let repaired = LineRepair::repaired("Gr\0\0e aus K\0ln", "  Grüße   aus Köln ");
    assert_eq!(repaired.as_deref(), Some("Grüße aus Köln"));
    assert_eq!(LineRepair::repaired("a\0b", "a b"), None);
}

#[test]
fn a_reread_that_changes_a_known_character_is_refused() {
    assert_eq!(LineRepair::repaired("K\0ln 50667", "Köln 50668"), None);
    assert_eq!(LineRepair::repaired("Herr M\0ller", "Frau Müller"), None);
}

#[test]
fn a_line_lost_completely_takes_any_word_of_up_to_that_many_characters_only_the_ink_check_is_left() {
    assert_eq!(LineRepair::repaired("\0\0\0", "Öl").as_deref(), Some("Öl"));
    assert_eq!(LineRepair::repaired("\0\0\0", "Öl ab"), None);
}
