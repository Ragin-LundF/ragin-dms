use dms_domain::library::{FileNameViolation, RelativePath};
use dms_domain::settings::InboxSettings;

use crate::test_data::path;

#[test]
fn the_inbox_path_must_be_a_valid_folder_below_the_root() {
    assert_eq!(InboxSettings::path_violation(&path("Inbox/Scanner")), None);
    assert_eq!(InboxSettings::path_violation(&RelativePath::root()), Some(FileNameViolation::Empty));
    assert_eq!(InboxSettings::path_violation(&path(".dms-text/x")), Some(FileNameViolation::LeadingDotOrSpace));
    assert_eq!(InboxSettings::path_violation(&path("CON")), Some(FileNameViolation::ReservedName));
}

#[test]
fn only_documents_below_the_inbox_count_and_only_while_it_is_on() {
    let inbox = InboxSettings { enabled: true, ..InboxSettings::default() };

    assert!(inbox.contains(&path("Inbox/a.pdf")));
    assert!(!inbox.contains(&path("Inboxes/a.pdf")));
    assert!(!InboxSettings { enabled: false, ..inbox }.contains(&path("Inbox/a.pdf")));
}
