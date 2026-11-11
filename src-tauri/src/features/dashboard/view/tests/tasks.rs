use dms_domain::change::{ContentConflict, MissingFile, ModifiedFile, MovedFile};
use dms_domain::classification::{ClassificationSuggestion, SuggestedCategory, SuggestedTag};
use dms_domain::taxonomy::TagName;

use super::*;
use crate::features::dashboard::dto::{DashboardAction, DashboardButtonStyle, MoveChoice};
use crate::session::Lane;

fn missing(present_in: &[&str]) -> MissingFile {
    MissingFile {
        document_id: id("d1"),
        path: path("Bills/a.pdf"),
        missing_in: std::iter::once(folder_id(PRIMARY)).collect(),
        present_in: present_in.iter().map(|id| folder_id(id)).collect(),
    }
}

fn library() -> Library {
    Library::new().category("c-1", "Bills").document("d1", "Bills/a.pdf", "2025-01-01T00:00:00Z", categorized("c-1"))
}

#[test]
fn groups_follow_the_fixed_order_and_a_new_copy_of_a_document_is_a_duplicate() {
    let library = library();
    // Seed 1 is the content of d1, so this new file is a copy of it.
    let copy = new_file("Inbox/a copy.pdf", Some(1), &[PRIMARY]);
    let fresh = new_file("fresh.pdf", Some(99), &[PRIMARY, SYNC]);
    let built = dashboard(vec![
        OpenTask::AdoptNewFile(fresh),
        OpenTask::AdoptNewFile(copy),
        OpenTask::ReviewSuggestion(ClassificationSuggestion::empty(id("d1"), "model")),
    ]);
    let view = present(library.catalog(), Some(&built));
    let order: Vec<TaskGroupKind> = view.groups.iter().map(|group| group.group).collect();
    assert_eq!(order, vec![TaskGroupKind::Suggestions, TaskGroupKind::NewFiles, TaskGroupKind::Duplicates]);
    let duplicate = only_row(&view, TaskGroupKind::Duplicates);
    assert_eq!(duplicate.title, "Copy of a document: Inbox/a copy.pdf");
    assert_eq!(duplicate.detail, "In Main, 2.0 KB; same size and checksum as Bills/a.pdf");
    assert_eq!(labels(&duplicate.buttons), vec!["Move copy to trash…", "Adopt anyway", "Dismiss"]);
    let fresh = only_row(&view, TaskGroupKind::NewFiles);
    assert_eq!(fresh.title, "New file fresh.pdf");
    assert_eq!(fresh.detail, "In Main, Cloud, 2.0 KB");
    assert_eq!(labels(&fresh.opens), vec!["Open"]);
    assert_eq!(fresh.preview.as_ref().map(|file| file.folder_id.as_str()), Some(PRIMARY));
}

#[test]
fn a_new_file_that_was_not_read_cannot_be_adopted_and_trashing_asks_for_its_folders() {
    let built = dashboard(vec![OpenTask::AdoptNewFile(new_file("x.pdf", None, &[SYNC]))]);
    let view = present(Library::new().catalog(), Some(&built));
    let row = only_row(&view, TaskGroupKind::NewFiles);
    let adopt = named(&row.buttons, "Adopt");
    assert_eq!(adopt.disabled_reason.as_deref(), Some("The file was not read yet; scan again"));
    assert_eq!(adopt.lane, Some(Lane::Library));
    let trash = named(&row.buttons, "Move to trash…").confirm.as_ref().expect("asks first");
    assert_eq!(trash.title, "Move to trash");
    assert_eq!(trash.affected, vec!["Cloud: x.pdf"]);
    assert_eq!(labels(&group(&view, TaskGroupKind::NewFiles).bulk_actions), vec!["Adopt all (0)"]);
}

#[test]
fn adopt_all_counts_one_readable_file_per_content() {
    let built = dashboard(vec![
        OpenTask::AdoptNewFile(new_file("a.pdf", Some(7), &[PRIMARY])),
        OpenTask::AdoptNewFile(new_file("b.pdf", Some(7), &[PRIMARY])),
        OpenTask::AdoptNewFile(new_file("c.pdf", None, &[PRIMARY])),
    ]);
    let view = present(Library::new().catalog(), Some(&built));
    assert_eq!(labels(&group(&view, TaskGroupKind::NewFiles).bulk_actions), vec!["Adopt all (1)"]);
}

#[test]
fn a_missing_file_restores_only_from_a_folder_that_has_it() {
    let built = dashboard(vec![OpenTask::RestoreOrDelete(missing(&[]))]);
    let view = present(library().catalog(), Some(&built));
    let row = only_row(&view, TaskGroupKind::Missing);
    assert_eq!(row.title, "Missing Bills/a.pdf");
    assert_eq!(row.detail, "Missing in Main; still in no reachable folder");
    assert_eq!(
        named(&row.buttons, "Restore").disabled_reason.as_deref(),
        Some("No reachable folder still has this file")
    );
    let delete = named(&row.buttons, "Move to trash…").confirm.as_ref().expect("asks first");
    assert_eq!(delete.title, "Delete everywhere");
    assert_eq!(delete.affected, vec!["Main: Bills/a.pdf", "Cloud: Bills/a.pdf"]);
    assert!(row.opens.is_empty());
    assert_eq!(labels(&group(&view, TaskGroupKind::Missing).bulk_actions), vec!["Restore all (0)"]);
}

#[test]
fn a_changed_file_keeps_the_new_or_restores_the_previous_version() {
    let change = ModifiedFile {
        document_id: id("d1"),
        path: path("Bills/a.pdf"),
        folder_id: folder_id(SYNC),
        size: 1,
        content_hash: None,
    };
    let built = dashboard(vec![OpenTask::ReviewModification(change)]);
    let view = present(library().catalog(), Some(&built));
    let row = only_row(&view, TaskGroupKind::Modified);
    assert_eq!(row.detail, "The content in Cloud differs from the catalog");
    assert_eq!(labels(&row.opens), vec!["Open new version", "Open previous version"]);
    let keep = named(&row.buttons, "Keep new content…");
    assert_eq!(keep.action, DashboardAction::KeepVersion { task_key: row.key.clone(), folder_id: SYNC.to_owned() });
    let question = keep.confirm.as_ref().expect("asks first");
    assert_eq!(question.title, "Keep the version of Cloud");
    assert_eq!(question.affected, vec!["Main: Bills/a.pdf"]);
    let restore = named(&row.buttons, "Restore previous…");
    assert_eq!(
        restore.action,
        DashboardAction::KeepVersion { task_key: row.key.clone(), folder_id: PRIMARY.to_owned() }
    );
}

#[test]
fn a_moved_file_offers_both_names() {
    let change = MovedFile {
        document_id: id("d1"),
        path: path("Bills/b.pdf"),
        previous_path: path("Bills/a.pdf"),
        folder_id: folder_id(SYNC),
    };
    let built = dashboard(vec![OpenTask::ConfirmMove(change)]);
    let view = present(library().catalog(), Some(&built));
    let row = only_row(&view, TaskGroupKind::Moved);
    assert_eq!(row.title, "Other name in Cloud: Bills/b.pdf");
    assert_eq!(labels(&row.buttons), vec!["Rename to a.pdf here", "Use b.pdf everywhere", "Dismiss"]);
    assert_eq!(
        row.buttons[0].action,
        DashboardAction::ResolveMove { task_key: "moved:d1:f-sync".to_owned(), choice: MoveChoice::Undo }
    );
    assert_eq!(labels(&group(&view, TaskGroupKind::Moved).bulk_actions), vec!["Rename all to match (1)"]);
}

#[test]
fn a_conflict_only_in_the_primary_cannot_keep_both() {
    let change = ContentConflict {
        path: path("x.pdf"),
        content_hashes: std::iter::once((folder_id(PRIMARY), hash(5))).collect(),
    };
    let built = dashboard(vec![OpenTask::ResolveContentConflict(change)]);
    let view = present(Library::new().catalog(), Some(&built));
    let row = only_row(&view, TaskGroupKind::Conflicts);
    assert_eq!(row.detail, "1 versions in Main");
    assert_eq!(labels(&row.buttons), vec!["Keep Main…", "Keep both", "Dismiss"]);
    assert_eq!(
        named(&row.buttons, "Keep both").disabled_reason.as_deref(),
        Some("Only the primary folder has a version")
    );
    assert_eq!(labels(&row.opens), vec!["Open Main version"]);
}

#[test]
fn duplicate_copies_list_the_recommended_copy_first_and_carry_their_own_open_buttons() {
    let library = Library::new()
        .category("c-1", "Bills")
        .document_with("bare", "a.pdf", "2025-01-01T00:00:00Z", 3, nothing)
        .document_with("worked", "b.pdf", "2025-02-01T00:00:00Z", 3, categorized("c-1"));
    let built = dashboard(vec![OpenTask::ResolveDuplicates {
        content_hash: hash(3),
        document_ids: vec![id("bare"), id("worked")],
    }]);
    let view = present(library.catalog(), Some(&built));
    let row = only_row(&view, TaskGroupKind::Duplicates);
    assert_eq!(row.title, "2 copies of the same file");
    assert!(row.opens.is_empty());
    assert_eq!(labels(&row.buttons), vec!["Dismiss"]);
    assert_eq!(row.copies[0].path, "b.pdf");
    assert_eq!(row.copies[0].detail, "Recommended · Category: Bills");
    assert_eq!(row.copies[0].keep.style, DashboardButtonStyle::Tonal);
    assert_eq!(row.copies[1].detail, "Nothing assigned");
    assert_eq!(row.copies[1].keep.style, DashboardButtonStyle::Outlined);
    let keep = row.copies[0].keep.confirm.as_ref().expect("asks first");
    assert_eq!(keep.title, "Keep b.pdf");
    assert_eq!(keep.affected, vec!["Main: a.pdf", "Cloud: a.pdf"]);
    let remove_all = &group(&view, TaskGroupKind::Duplicates).bulk_actions[0];
    assert_eq!(remove_all.label, "Remove all duplicates (1)…");
    assert_eq!(remove_all.confirm.as_ref().expect("asks first").affected, vec!["a.pdf (copy of b.pdf)"]);
}

#[test]
fn a_suggestion_names_only_the_parts_it_proposes() {
    let mut suggestion = ClassificationSuggestion::empty(id("d1"), "model");
    suggestion.title = Some("Power bill".to_owned());
    suggestion.category = Some(SuggestedCategory::New { name: "Energy".to_owned(), parent_id: None });
    suggestion.tags = vec![SuggestedTag { name: TagName::of("home").expect("tag"), is_new: false }];
    suggestion.file_name = Some("power.pdf".to_owned());
    let built = dashboard(vec![OpenTask::ReviewSuggestion(suggestion)]);
    let view = present(library().catalog(), Some(&built));
    let row = only_row(&view, TaskGroupKind::Suggestions);
    assert_eq!(row.title, "Suggestion for Bills/a.pdf");
    assert_eq!(row.detail, "Title: Power bill · Category: Energy (new) · Tags: home · Rename → power.pdf");
    assert_eq!(labels(&row.buttons), vec!["Review", "Reject"]);
    assert_eq!(row.buttons[0].action, DashboardAction::Review { document_id: "d1".to_owned() });
}

#[test]
fn retention_and_deadlines_have_their_own_buttons() {
    let date = dms_domain::parse_iso_date("2026-03-01").expect("date");
    let built = dashboard(vec![
        OpenTask::RetentionDue { document_id: id("d1"), date },
        OpenTask::DeadlineDue {
            document_id: id("d1"),
            field_key: dms_domain::field::FieldKey::new("due").expect("key"),
            date,
        },
    ]);
    let view = present(library().catalog(), Some(&built));
    let retention = only_row(&view, TaskGroupKind::Retention);
    assert_eq!(retention.title, "Retention ended: Bills/a.pdf");
    assert_eq!(labels(&retention.buttons), vec!["Mark as done", "Move to trash…"]);
    assert_eq!(
        labels(&group(&view, TaskGroupKind::Retention).bulk_actions),
        vec!["Mark all as done (1)", "Move all to trash (1)…"]
    );
    let trash = retention.buttons[1].confirm.as_ref().expect("asks first");
    assert_eq!(trash.title, "Move 1 documents to the trash");
    let deadline = only_row(&view, TaskGroupKind::Deadlines);
    assert_eq!(deadline.title, "due on 2026-03-01: Bills/a.pdf");
    assert_eq!(labels(&deadline.buttons), vec!["Dismiss"]);
}

#[test]
fn failed_operations_and_folder_structure() {
    let built = dashboard(vec![
        OpenTask::RetryFailedOperations { folder_id: folder_id(SYNC), count: 2 },
        OpenTask::RestructureSuggested { folder: path("Scans"), file_count: 41 },
    ]);
    let view = present(Library::new().catalog(), Some(&built));
    let retry = only_row(&view, TaskGroupKind::FailedOperations);
    assert_eq!(retry.title, "2 operation(s) waiting for Cloud");
    assert_eq!(retry.buttons[0].action, DashboardAction::Retry);
    let restructure = only_row(&view, TaskGroupKind::Restructure);
    assert_eq!(restructure.title, "41 files without structure in Scans");
    assert_eq!(restructure.buttons[0].action, DashboardAction::Restructure { folder: "Scans".to_owned() });
}
