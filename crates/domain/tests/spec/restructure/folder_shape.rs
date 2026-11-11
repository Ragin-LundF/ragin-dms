use dms_domain::catalog::Catalog;
use dms_domain::document::DocumentMetadata;
use dms_domain::restructure::{FlatFolder, FolderShape};
use dms_domain::settings::InboxSettings;

use crate::test_data::{self, Applied, date, path};

fn catalog_with(folder: &str, count: i32, dated: bool) -> Catalog {
    (1..=count).fold(test_data::catalog_with(&[]), |catalog, index| {
        let document = test_data::document(&format!("{folder}-{index}"), &format!("{folder}/file-{index}.pdf"));
        let added = catalog.add_document(document.clone()).applied();
        if dated {
            let metadata =
                DocumentMetadata { document_date: Some(date(2020 + index % 3, 1, 1)), ..DocumentMetadata::default() };
            added.update_metadata(&document.id, metadata).applied()
        } else {
            added
        }
    })
}

fn analyse(catalog: &Catalog, inbox: &InboxSettings) -> Vec<FlatFolder> {
    FolderShape::analyse(catalog, inbox, FolderShape::DEFAULT_THRESHOLD)
}

#[test]
fn a_folder_with_more_files_than_the_threshold_and_no_subfolders_is_flat() {
    let catalog = catalog_with("scans", 41, false);

    assert_eq!(analyse(&catalog, &InboxSettings::default()), vec![FlatFolder { path: path("scans"), file_count: 41 }]);
}

#[test]
fn a_folder_with_subfolders_is_not_flat() {
    let catalog =
        catalog_with("scans", 41, false).add_document(test_data::document("deep", "scans/2024/deep.pdf")).applied();

    assert_eq!(analyse(&catalog, &InboxSettings::default()), [] as [dms_domain::restructure::FlatFolder; 0]);
}

#[test]
fn a_smaller_folder_that_splits_cleanly_by_year_is_offered() {
    assert_eq!(analyse(&catalog_with("letters", 12, true), &InboxSettings::default()).len(), 1);
    assert_eq!(
        analyse(&catalog_with("letters", 12, false), &InboxSettings::default()),
        [] as [dms_domain::restructure::FlatFolder; 0]
    );
}

#[test]
fn the_inbox_is_never_offered() {
    let catalog = catalog_with("Inbox", 50, false);

    assert_eq!(
        analyse(&catalog, &InboxSettings { enabled: true, ..InboxSettings::default() }),
        [] as [dms_domain::restructure::FlatFolder; 0]
    );
}
