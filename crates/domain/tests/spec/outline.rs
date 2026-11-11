use dms_domain::catalog::Catalog;
use dms_domain::document::Document;
use dms_domain::outline::FolderOutline;
use dms_domain::taxonomy::Category;
use indexmap::{IndexSet, indexset};

use crate::test_data::{self, Applied, category_id, path};

fn bills() -> Category {
    Category::new(category_id("bills"), "Bills", None, None).expect("valid")
}

fn catalog(paths: &[&str]) -> Catalog {
    let base = test_data::catalog_with(&[]).add_category(bills()).applied();
    paths.iter().enumerate().fold(base, |catalog, (index, raw)| {
        let category_id = raw.starts_with("Home/").then(|| bills().id);
        let document = Document { category_id, ..test_data::document(&format!("d{index}"), raw) };
        catalog.add_document(document).applied()
    })
}

#[test]
fn folders_carry_counts_the_most_common_category_and_sample_titles_without_the_inbox() {
    let outline = FolderOutline::of(
        &catalog(&["Home/Power/a.pdf", "Home/Water/b.pdf", "Inbox/new.pdf", "Car/c.pdf"]),
        vec![path("Inbox")],
        FolderOutline::DEFAULT_LIMIT,
    );

    assert_eq!(
        outline.folders.iter().map(|folder| folder.path.value()).collect::<Vec<_>>(),
        vec!["Car", "Home", "Home/Power", "Home/Water"]
    );
    let home = outline.folders.iter().find(|folder| folder.path.value() == "Home").expect("Home is shown");
    assert_eq!(home.document_count, 2);
    assert_eq!(home.top_category, Some(bills().id));
    assert_eq!(home.sample_titles.iter().map(String::as_str).collect::<IndexSet<_>>(), indexset! { "a.pdf", "b.pdf" });
}

#[test]
fn a_large_tree_is_cut_at_a_depth_and_the_cut_folders_count_what_they_hide() {
    let paths: Vec<String> = (1..=30).flat_map(|top| (1..=10).map(move |sub| format!("T{top}/S{sub}/f.pdf"))).collect();
    let raw: Vec<&str> = paths.iter().map(String::as_str).collect();

    let outline = FolderOutline::of(&catalog(&raw), Vec::new(), 200);

    assert_eq!(outline.folders.len(), 30);
    assert_eq!(outline.folders[0].hidden_subfolders, 10);
}

#[test]
fn new_folders_are_allowed_directly_below_a_known_folder_never_in_the_inbox() {
    let outline = FolderOutline::of(&catalog(&["Home/a.pdf"]), vec![path("Inbox")], FolderOutline::DEFAULT_LIMIT);

    assert!(outline.can_create(&path("Home/Garden")));
    assert!(outline.can_create(&path("Garden")));
    assert!(!outline.can_create(&path("Away/Garden")));
    assert!(!outline.can_create(&path("Inbox/Sorted")));
    assert!(!outline.can_create(&path("Inbox")));
}
