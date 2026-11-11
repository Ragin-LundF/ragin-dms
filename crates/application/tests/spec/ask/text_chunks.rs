use dms_application::ask::TextChunks;

#[test]
fn a_short_text_is_one_passage_and_an_empty_text_one_empty_passage() {
    let texts = |text: &str| TextChunks::of(text).into_iter().map(|chunk| chunk.text).collect::<Vec<_>>();
    assert_eq!(texts("  Sample bill "), vec!["Sample bill"]);
    assert_eq!(texts(" "), vec![""]);
}

#[test]
fn a_long_text_is_cut_at_blanks_into_overlapping_passages_that_cover_all_of_it() {
    let text = (1..=2_000).map(|n| format!("word{n}")).collect::<Vec<_>>().join(" ");

    let chunks = TextChunks::of(&text);

    assert!(chunks.len() > 1);
    assert!(chunks.iter().all(|chunk| chunk.text.len() <= TextChunks::SIZE));
    assert_eq!(chunks.first().expect("a chunk").start, 0);
    assert_eq!(chunks.last().expect("a chunk").end, text.len());
    for pair in chunks.windows(2) {
        let (first, second) = (&pair[0], &pair[1]);
        assert!(second.start < first.end, "passages overlap");
        assert!(first.end == text.len() || text.as_bytes()[first.end] == b' ', "cut at a blank");
    }
}

#[test]
fn a_huge_text_is_capped() {
    let text = "x".repeat(TextChunks::SIZE * (TextChunks::MAX_CHUNKS + 10));

    assert_eq!(TextChunks::of(&text).len(), TextChunks::MAX_CHUNKS);
}

#[test]
fn offsets_count_utf16_units_like_kotlin() {
    let text = format!("{} {}", "ä".repeat(10), "€".repeat(5));

    let chunk = TextChunks::of(&text).pop().expect("one chunk");

    assert_eq!((chunk.start, chunk.end), (0, 16));
}
