use dms_domain::library::{FolderLocation, FolderRole, LibraryFolder, LibraryFolderId, LibraryFolders};

fn folder(id: &str, role: FolderRole) -> LibraryFolder {
    LibraryFolder::new(
        LibraryFolderId::new(id).expect("valid"),
        id,
        FolderLocation::new(format!("/tmp/{id}")).expect("valid"),
        role,
    )
    .expect("valid folder")
}

fn main_folder() -> LibraryFolder {
    folder("main", FolderRole::Primary)
}

fn backup() -> LibraryFolder {
    folder("backup", FolderRole::Sync)
}

fn share() -> LibraryFolder {
    folder("share", FolderRole::Sync)
}

fn folders() -> LibraryFolders {
    LibraryFolders::new(vec![main_folder(), backup()]).expect("valid folders")
}

#[test]
fn primary_and_sync_folders_are_separated() {
    assert_eq!(folders().primary(), &main_folder());
    assert_eq!(folders().sync_folders(), vec![&backup()]);
}

#[test]
fn exactly_one_primary_is_required() {
    assert!(LibraryFolders::new(vec![backup()]).is_err());
    let second = LibraryFolder { id: LibraryFolderId::new("second").expect("valid"), ..main_folder() };
    assert!(LibraryFolders::new(vec![main_folder(), second]).is_err());
}

#[test]
fn a_location_can_only_be_used_once() {
    assert!(folders().add(LibraryFolder { location: backup().location, ..share() }).is_err());
    assert!(folders().contains(&backup().location));
    assert!(!folders().contains(&share().location));
}

#[test]
fn sync_folders_can_be_added_and_removed_but_the_primary_cannot_be_removed() {
    let extended = folders().add(share()).expect("a new sync folder");

    assert_eq!(extended.sync_folders(), vec![&backup(), &share()]);
    assert_eq!(extended.remove(&backup().id).expect("a sync folder").sync_folders(), vec![&share()]);
    assert!(extended.remove(&main_folder().id).is_err());
}

#[test]
fn choosing_another_primary_demotes_the_old_one() {
    let switched = folders().with_primary(&backup().id).expect("a known folder");

    assert_eq!(switched.primary().id, backup().id);
    assert_eq!(switched.sync_folders().iter().map(|it| it.id.clone()).collect::<Vec<_>>(), vec![main_folder().id]);
}
