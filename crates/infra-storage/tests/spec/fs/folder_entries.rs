use std::fs;

use dms_application::port::storage::{DocumentStorage, FolderEntries, StorageFailure};
use indexmap::IndexSet;
use unicode_normalization::UnicodeNormalization;

use super::Folders;
use crate::support::{failure_of, path, value_of, write};

#[test]
fn every_folder_also_an_empty_one_and_every_file_below_a_folder_are_listed_the_tools_files_left_out() {
    let folders = Folders::new();
    let root = &folders.primary_root;
    write(&root.join("Home/Energy/power.pdf"), "power");
    fs::create_dir_all(root.join("Home/Empty")).expect("folder");
    write(&root.join("Home/.DS_Store"), "x");
    write(&root.join("Home/Energy/.dms-tmp-1234"), "x");
    write(&root.join("Other/letter.pdf"), "letter");

    let entries = value_of(folders.storage.list_entries(&folders.primary, &path("Home")));

    assert_eq!(
        entries,
        FolderEntries {
            folders: IndexSet::from([path("Home/Empty"), path("Home/Energy")]),
            files: IndexSet::from([path("Home/Energy/power.pdf")]),
        }
    );
}

#[test]
fn names_come_back_composed_and_a_missing_folder_is_not_found() {
    let folders = Folders::new();
    let decomposed: String = "Bücher".nfd().collect();
    write(&folders.primary_root.join(decomposed).join("a.pdf"), "a");

    let all = value_of(folders.storage.list_entries(&folders.primary, &path("")));
    let missing = folders.storage.list_entries(&folders.primary, &path("Nowhere"));

    assert_eq!(all.folders, IndexSet::from([path("Bücher")]));
    assert_eq!(failure_of(missing), Some(StorageFailure::NotFound));
}

#[test]
fn listing_the_folders_finds_every_folder_at_any_depth_also_empty_ones_and_no_files() {
    let folders = Folders::new();
    let root = &folders.primary_root;
    write(&root.join("Home/Energy/Old/gas.pdf"), "gas");
    fs::create_dir_all(root.join("Home/Empty")).expect("folder");
    fs::create_dir_all(root.join(".Trashes/kept")).expect("folder");
    write(&root.join("letter.pdf"), "letter");

    let listed = value_of(folders.storage.list_folders(&folders.primary));

    let expected: IndexSet<_> =
        ["Home", "Home/Energy", "Home/Energy/Old", "Home/Empty"].into_iter().map(path).collect();
    assert_eq!(listed, expected);
}
