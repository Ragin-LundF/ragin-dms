use std::sync::Arc;

use dms_application::library::{ConfigureLibrary, ConfigureLibraryOutcome, LibraryCommand};
use dms_application::port::state::SettingsRepository;
use dms_domain::library::{FolderLocation, FolderRole, LibraryFolders};

use crate::support::fake::state::InMemorySettings;
use crate::support::fake::system::SequentialIds;

struct Setup {
    settings: Arc<InMemorySettings>,
    configure: ConfigureLibrary,
}

fn setup() -> Setup {
    let settings = Arc::new(InMemorySettings::default());
    let configure = ConfigureLibrary::new(settings.clone(), Arc::new(SequentialIds::default()));
    Setup { settings, configure }
}

fn primary_location() -> FolderLocation {
    FolderLocation::new("/library/main").expect("valid location")
}

fn backup_location() -> FolderLocation {
    FolderLocation::new("/library/backup").expect("valid location")
}

fn create_primary(setup: &Setup) {
    setup.configure.run(&LibraryCommand::CreatePrimary { display_name: "Main".into(), location: primary_location() });
}

#[track_caller]
fn configured(outcome: ConfigureLibraryOutcome) -> LibraryFolders {
    match outcome {
        ConfigureLibraryOutcome::Configured(folders) => folders,
        other => panic!("expected Configured, got {other:?}"),
    }
}

#[test]
fn the_first_folder_becomes_the_primary_and_further_folders_sync_folders() {
    let setup = setup();
    create_primary(&setup);

    let outcome = setup
        .configure
        .run(&LibraryCommand::AddSyncFolder { display_name: "Backup".into(), location: backup_location() });

    let folders = configured(outcome);
    assert_eq!(folders.primary().location, primary_location());
    let sync_locations: Vec<_> = folders.sync_folders().iter().map(|folder| folder.location.clone()).collect();
    assert_eq!(sync_locations, vec![backup_location()]);
    assert_eq!(setup.settings.load_library_folders(), Ok(Some(folders)));
}

#[test]
fn a_location_can_only_be_used_once() {
    let setup = setup();
    create_primary(&setup);

    let outcome = setup
        .configure
        .run(&LibraryCommand::AddSyncFolder { display_name: "Again".into(), location: primary_location() });

    assert_eq!(outcome, ConfigureLibraryOutcome::LocationAlreadyUsed);
}

#[test]
fn the_primary_cannot_be_removed_but_another_folder_can_become_the_primary() {
    let setup = setup();
    create_primary(&setup);
    let added = setup
        .configure
        .run(&LibraryCommand::AddSyncFolder { display_name: "Backup".into(), location: backup_location() });
    let folders = configured(added);
    let backup = folders.sync_folders()[0].clone();

    let removal = setup.configure.run(&LibraryCommand::RemoveSyncFolder(folders.primary().id.clone()));
    let switched = setup.configure.run(&LibraryCommand::ChoosePrimary(backup.id.clone()));

    assert_eq!(removal, ConfigureLibraryOutcome::PrimaryCannotBeRemoved);
    let result = configured(switched);
    assert_eq!(result.find(&backup.id).map(|folder| folder.role), Some(FolderRole::Primary));
}
