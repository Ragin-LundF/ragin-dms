use chrono::NaiveDate;

use crate::support::TestLibrary;
use crate::support::test_data::{date, path};

fn date_of(library: &TestLibrary, raw_path: &str) -> Option<NaiveDate> {
    library.catalog().find_by_path(&path(raw_path)).and_then(|document| document.metadata.document_date)
}

#[test]
fn new_files_get_the_date_of_their_name_else_of_their_creation_else_none() {
    let library = TestLibrary::new();
    library.open();
    library.folders.put(&library.primary, "15.03.2024 Sample.pdf", "alpha");
    library.folders.put(&library.primary, "created.pdf", "beta");
    library.folders.put(&library.primary, "unknown.pdf", "gamma");
    library.folders.set_creation_day("created.pdf", date(2019, 6, 1));

    library.scanned(true);

    assert_eq!(date_of(&library, "15.03.2024 Sample.pdf"), Some(date(2024, 3, 15)));
    assert_eq!(date_of(&library, "created.pdf"), Some(date(2019, 6, 1)));
    assert_eq!(date_of(&library, "unknown.pdf"), None);
}

#[test]
fn documents_without_a_date_get_one_later_dates_already_set_stay() {
    let library = TestLibrary::new();
    library.open();
    library.folders.put(&library.primary, "2021-02-03 Sample.pdf", "alpha");
    library.folders.put(&library.primary, "other.pdf", "beta");
    library.scanned(true);
    library.folders.set_creation_day("other.pdf", date(2018, 1, 1));

    let dated = library.detect_document_dates().run();

    assert_eq!(dated, 1);
    assert_eq!(date_of(&library, "other.pdf"), Some(date(2018, 1, 1)));
    assert_eq!(date_of(&library, "2021-02-03 Sample.pdf"), Some(date(2021, 2, 3)));
}
