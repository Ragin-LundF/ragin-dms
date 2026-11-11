use chrono::Utc;

use super::*;
use crate::features::dashboard::dto::{ChartPointDto, StatLink};
use crate::features::dashboard::view::DashboardInput;

fn point(id: &str, value: u32) -> ChartPointDto {
    ChartPointDto { id: id.to_owned(), label: id.to_owned(), value }
}

#[test]
fn the_header_names_documents_size_folders_and_the_last_scan() {
    let library = Library::new().document("d1", "a.pdf", "2026-03-01T10:00:00Z", nothing);
    let built = dashboard(Vec::new());
    let view = present_dashboard(&DashboardInput {
        catalog: library.catalog(),
        dashboard: Some(&built),
        foreign: &[],
        last_scan_at: Some(instant("2026-03-09T08:05:00Z")),
        ai_enabled: false,
        scan_count: None,
        now: now(),
        zone: &Utc,
    });
    assert_eq!(view.header_line, "1 documents · 2.0 KB · 2 library folders · last scan 09.03. 08:05");
    assert_eq!(
        present(library.catalog(), None).header_line,
        "1 documents · 2.0 KB · 0 library folders · not scanned yet"
    );
}

#[test]
fn the_documents_figure_shows_the_growth_of_the_last_30_days() {
    let library = Library::new()
        .document("old", "old.pdf", "2025-01-01T00:00:00Z", nothing)
        // Exactly 24 h before now counts for "one day ago", not for the calendar day before.
        .document("new", "new.pdf", "2026-03-09T12:00:00Z", nothing);
    let documents = &present(library.catalog(), None).stats[0];
    assert_eq!(documents.caption, "+1 in the last 30 days");
    assert_eq!(documents.count, Some(2));
    assert_eq!(documents.trend.len(), 30);
    assert_eq!(&documents.trend[27..], &[1, 2, 2]);
}

#[test]
fn without_recent_additions_the_documents_figure_draws_the_years() {
    let library = Library::new().document("old", "old.pdf", "2025-01-01T00:00:00Z", |document| {
        document.metadata.document_date = dms_domain::parse_iso_date("2024-05-01");
    });
    let view = present(library.catalog(), None);
    assert_eq!(view.stats[0].caption, "None added lately");
    assert_eq!(view.stats[0].trend, vec![1, 0, 0]);
    assert_eq!(view.by_year, vec![point("2024", 1), point("2025", 0), point("2026", 0)]);
}

#[test]
fn categorized_shows_the_share_and_links_to_the_uncategorized() {
    let library = Library::new()
        .category("c-1", "Bills")
        .document("d1", "a.pdf", "2025-01-01T00:00:00Z", categorized("c-1"))
        .document("d2", "b.pdf", "2025-01-01T00:00:00Z", categorized("c-1"))
        .document("d3", "c.pdf", "2025-01-01T00:00:00Z", nothing);
    let column = &present(library.catalog(), None).stats[1];
    assert_eq!(column.value, "67 %");
    assert_eq!(column.caption, "1 without a category");
    assert_eq!(column.link, Some(StatLink::Uncategorized));
    let all_categorized =
        Library::new().category("c-1", "Bills").document("d1", "a.pdf", "2025-01-01T00:00:00Z", categorized("c-1"));
    assert_eq!(present(all_categorized.catalog(), None).stats[1].link, None);
}

#[test]
fn the_inbox_figure_is_off_until_the_inbox_is_switched_on() {
    let off = Library::new();
    let column = &present(off.catalog(), None).stats[2];
    assert_eq!(
        (column.value.as_str(), column.caption.as_str(), column.link),
        ("Off", "Switched off in the settings", None)
    );
    let on = Library::new().inbox("Inbox", false).document("d1", "Inbox/scan.pdf", "2025-01-01T00:00:00Z", nothing);
    let column = &present(on.catalog(), None).stats[2];
    assert_eq!(
        (column.value.as_str(), column.caption.as_str(), column.link),
        ("1", "Waiting to be filed", Some(StatLink::Inbox))
    );
}

#[test]
fn years_older_than_eight_fold_into_the_first_bar() {
    let dated = |date: &'static str| {
        move |document: &mut Document| document.metadata.document_date = dms_domain::parse_iso_date(date)
    };
    let library = Library::new()
        .document("d1", "a.pdf", "2026-01-01T00:00:00Z", dated("2010-01-01"))
        .document("d2", "b.pdf", "2026-01-01T00:00:00Z", dated("2019-06-01"))
        .document("d3", "c.pdf", "2026-01-01T00:00:00Z", dated("2020-06-01"))
        // Without a document date, the year it was added counts.
        .document("d4", "d.pdf", "2026-01-01T00:00:00Z", nothing);
    let years = present(library.catalog(), None).by_year;
    assert_eq!(years.len(), 8);
    assert_eq!(years[0], ChartPointDto { id: "≤2019".to_owned(), label: "≤2019".to_owned(), value: 2 });
    assert_eq!(years[1], point("2020", 1));
    assert_eq!(years[7], point("2026", 1));
}

#[test]
fn rankings_are_by_count_then_by_name_ignoring_case_and_correspondents_by_exact_spelling() {
    let correspondent =
        |name: &'static str| move |document: &mut Document| document.metadata.correspondent = Some(name.to_owned());
    let library = Library::new()
        .category("c-b", "bills")
        .category("c-a", "Archive")
        .document("d1", "1.pdf", "2025-01-01T00:00:00Z", categorized("c-b"))
        .document("d2", "2.pdf", "2025-01-01T00:00:00Z", categorized("c-a"))
        .document("d3", "3.pdf", "2025-01-01T00:00:00Z", correspondent("ACME"))
        .document("d4", "4.pdf", "2025-01-01T00:00:00Z", correspondent("Acme"))
        .document("d5", "5.pdf", "2025-01-01T00:00:00Z", correspondent("Acme"));
    let view = present(library.catalog(), None);
    let names: Vec<&str> = view.categories.iter().map(|point| point.label.as_str()).collect();
    assert_eq!(names, vec!["Archive", "bills", "Uncategorized"]);
    assert_eq!(view.categories[2].id, "");
    assert_eq!(view.categories[2].value, 3);
    let correspondents: Vec<(&str, u32)> =
        view.correspondents.iter().map(|point| (point.label.as_str(), point.value)).collect();
    assert_eq!(correspondents, vec![("Acme", 2), ("ACME", 1)]);
}

#[test]
fn tags_list_the_most_used_first() {
    let library = Library::new().document("d1", "1.pdf", "2025-01-01T00:00:00Z", tagged(&["tax", "car"])).document(
        "d2",
        "2.pdf",
        "2025-01-01T00:00:00Z",
        tagged(&["car"]),
    );
    let tags: Vec<(String, u32)> =
        present(library.catalog(), None).tags.into_iter().map(|point| (point.label, point.value)).collect();
    assert_eq!(tags, vec![("car".to_owned(), 2), ("tax".to_owned(), 1)]);
}

#[test]
fn recently_added_shows_the_six_newest_with_folder_category_and_relative_date() {
    let mut library = Library::new().category("c-1", "Bills");
    for day in 1..=7 {
        library = library.document(
            &format!("d{day}"),
            &format!("Folder/{day}.pdf"),
            &format!("2026-03-0{day}T09:00:00Z"),
            nothing,
        );
    }
    let library = library.document("top", "top.pdf", "2026-03-09T09:00:00Z", categorized("c-1"));
    let recent = present(library.catalog(), None).recent;
    assert_eq!(recent.len(), 6);
    assert_eq!(recent[0].title, "top.pdf");
    assert_eq!(recent[0].subtitle, "Top level · Bills");
    assert_eq!(recent[0].added, "yesterday");
    assert_eq!(recent[0].added_on, "2026-03-09");
    assert_eq!(recent[1].subtitle, "Folder");
}

#[test]
fn the_ai_band_counts_readable_documents_without_tags_or_category_and_no_waiting_suggestion() {
    let library = Library::new()
        .document("d1", "a.pdf", "2025-01-01T00:00:00Z", tagged(&["car"]))
        .document("d2", "b.pdf", "2025-01-01T00:00:00Z", nothing)
        // Plain text is never sent to the AI.
        .document("d3", "notes.txt", "2025-01-01T00:00:00Z", nothing);
    let band = present(library.catalog(), None).ai_band.expect("a band");
    assert_eq!(band.headline, "1 document has no tags yet, 2 have no category");
    assert_eq!(band.classify_label.as_deref(), Some("Classify 2"));
    assert_eq!(band.tag_label.as_deref(), Some("Tag 1 with AI"));
    let suggestion = dms_domain::classification::ClassificationSuggestion::empty(id("d2"), "model");
    let built = dashboard(vec![OpenTask::ReviewSuggestion(suggestion)]);
    let band = present(library.catalog(), Some(&built)).ai_band.expect("a band");
    assert_eq!(band.headline, "1 document has no category");
    assert_eq!(band.tag_label, None);
}

#[test]
fn folders_show_health_primary_and_pending_operations() {
    let mut built = dashboard(Vec::new());
    built.folders[0].pending_operations = 0;
    built.folders[1].reachable = Some(false);
    built.folders[1].pending_operations = 3;
    let view = present(Library::new().catalog(), Some(&built));
    assert_eq!(view.folders[0].title, "Main · primary");
    assert_eq!(view.folders[0].status, "reachable");
    assert_eq!(view.folders[1].title, "Cloud");
    assert_eq!(view.folders[1].status, "offline, 3 pending");
}
