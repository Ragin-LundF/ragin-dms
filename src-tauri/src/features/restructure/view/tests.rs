#![allow(clippy::default_trait_access)] // the IndexSet fields are built without naming indexmap, not a dependency here
use chrono::{DateTime, Utc};
use dms_application::restructure::RestructureOutcome;
use dms_domain::catalog::{Catalog, CatalogUpdate, LibraryId};
use dms_domain::document::{ContentHash, Document, DocumentId};
use dms_domain::library::{LibraryFolderId, RelativePath};
use dms_domain::query::{DocumentQuery, SavedSearch, SavedSearchId};
use dms_domain::restructure::{PlannedMove, RestructurePlan, SkipReason, SkippedDocument};

use super::*;

fn path(raw: &str) -> RelativePath {
    RelativePath::of(raw).expect("valid path")
}

fn id(raw: &str) -> DocumentId {
    DocumentId::new(raw).expect("valid id")
}

fn planned(document: &str, from: &str, to: &str) -> PlannedMove {
    PlannedMove { document_id: id(document), from: path(from), to: path(to) }
}

fn catalog(paths: &[&str]) -> Catalog {
    let added: DateTime<Utc> = "2024-01-01T10:00:00Z".parse().expect("instant");
    paths.iter().enumerate().fold(Catalog::empty(LibraryId::new("lib").expect("id")), |catalog, (index, raw)| {
        let hash = ContentHash::new(format!("{index:064x}")).expect("hash");
        let document = Document::new(id(&format!("d{index}")), path(raw), hash, 10, added).expect("document");
        match catalog.add_document(document).expect("valid") {
            CatalogUpdate::Applied(next) => next,
            CatalogUpdate::Rejected(reason) => panic!("rejected: {reason:?}"),
        }
    })
}

fn plan(moves: Vec<PlannedMove>) -> RestructurePlan {
    RestructurePlan {
        template: "{year}".to_owned(),
        moves,
        unchanged: Vec::new(),
        skipped: Vec::new(),
        renamed_for_clash: Default::default(),
    }
}

#[test]
fn scopes_list_whole_library_then_smart_folders_then_folders_ignoring_case() {
    let mut catalog = catalog(&["b/x.pdf", "A/sub/y.pdf", "a2/z.pdf", "top.pdf"]);
    let search =
        SavedSearch::new(SavedSearchId::new("s1").expect("id"), "Invoices", DocumentQuery::default()).expect("search");
    let mut settings = catalog.settings().clone();
    settings.saved_searches.push(search);
    catalog = match catalog.change_settings(settings).expect("valid") {
        CatalogUpdate::Applied(next) => next,
        CatalogUpdate::Rejected(reason) => panic!("rejected: {reason:?}"),
    };

    let labels: Vec<String> = scope_options(&catalog).into_iter().map(|option| option.label).collect();

    assert_eq!(labels, ["Whole library", "Smart folder: Invoices", "A", "A/sub", "a2", "b"]);
}

#[test]
fn smart_folder_scope_resolves_by_id_and_a_vanished_one_is_refused() {
    let catalog = catalog(&["x.pdf"]);

    let missing = resolve_scope(&catalog, &RestructureScopeDto::SmartFolder { id: "gone".to_owned() });
    let whole = resolve_scope(&catalog, &RestructureScopeDto::Folder { path: String::new() });

    assert_eq!(missing.map_err(|error| error.code).err().as_deref(), Some("invalid_input"));
    assert_eq!(whole.ok(), Some(RestructureScope::Folder(RelativePath::root())));
}

#[test]
fn plan_groups_moves_by_target_folder_sorted_with_three_samples() {
    let mut planned_plan = plan(vec![
        planned("1", "in/a.pdf", "Tax/2024/a.pdf"),
        planned("2", "in/b.pdf", "Bank/b.pdf"),
        planned("3", "in/c.pdf", "Tax/2024/c.pdf"),
        planned("4", "in/d.pdf", "Tax/2024/d.pdf"),
        planned("5", "in/e.pdf", "Tax/2024/e (2).pdf"),
        planned("6", "in/f.pdf", "f.pdf"),
    ]);
    planned_plan.unchanged = vec![id("7")];
    planned_plan.renamed_for_clash.insert(id("5"));
    planned_plan.skipped = vec![
        SkippedDocument { document_id: id("8"), path: path("x.pdf"), reason: SkipReason::MissingDate },
        SkippedDocument { document_id: id("9"), path: path("y.pdf"), reason: SkipReason::InInbox },
        SkippedDocument { document_id: id("10"), path: path("z.pdf"), reason: SkipReason::MissingDate },
    ];

    let dto = present_plan(&planned_plan);

    let folders: Vec<(&str, u32)> = dto.groups.iter().map(|group| (group.folder.as_str(), group.count)).collect();
    assert_eq!(folders, [("", 1), ("Bank", 1), ("Tax/2024", 4)]);
    assert_eq!(dto.groups[2].samples, ["in/a.pdf → a.pdf", "in/c.pdf → c.pdf", "in/d.pdf → d.pdf"]);
    assert_eq!(dto.unchanged, 1);
    assert_eq!(dto.skipped, ["2 without date", "1 in the inbox"]);
    assert_eq!(dto.renamed, ["e.pdf → Tax/2024/e (2).pdf"]);
}

#[test]
fn unticked_folders_leave_apply_and_the_confirmation_lists_twelve_moves() {
    let moves: Vec<PlannedMove> =
        (0..15).map(|n| planned(&n.to_string(), &format!("in/{n}.pdf"), &format!("Keep/{n}.pdf"))).collect();
    let mut all = moves.clone();
    all.push(planned("x", "in/x.pdf", "Drop/x.pdf"));

    let chosen = chosen_plan(&plan(all), &["Keep".to_owned()]);
    let preview = apply_preview(&chosen);

    assert_eq!(preview.moves, 15);
    assert_eq!(preview.affected.len(), 13);
    assert_eq!(preview.affected[0], "in/0.pdf → Keep/0.pdf");
    assert_eq!(preview.affected[12], "… and 3 more");
}

#[test]
fn applied_outcome_joins_its_parts_and_is_an_error_when_moves_failed() {
    let failed = planned("1", "a/x.pdf", "b/x.pdf");
    let mut outcome = RestructureOutcome::Applied {
        moved: 4,
        failed: vec![failed],
        not_catalogued: Vec::new(),
        skipped: 2,
        pending_folders: Default::default(),
        record: Some("20260101-101010".to_owned()),
        empty_folders: vec![path("a")],
        cancelled: false,
    };
    if let RestructureOutcome::Applied { pending_folders, .. } = &mut outcome {
        pending_folders.insert(LibraryFolderId::new("f2").expect("id"));
    }

    let report = report_of(&outcome);

    assert_eq!(
        report.message,
        UiMessage::error(
            "Moved 4 documents; 1 could not be moved; 2 changed since and stayed; 1 folders wait in the journal"
        )
    );
    assert_eq!(report.log_errors, ["Not moved: a/x.pdf"]);
    assert_eq!(report.empty_folders, ["a"]);
}

#[test]
fn a_clean_run_is_an_info_message() {
    let outcome = RestructureOutcome::Applied {
        moved: 3,
        failed: Vec::new(),
        not_catalogued: Vec::new(),
        skipped: 0,
        pending_folders: Default::default(),
        record: None,
        empty_folders: Vec::new(),
        cancelled: false,
    };

    assert_eq!(report_of(&outcome).message, UiMessage::info("Moved 3 documents"));
}
