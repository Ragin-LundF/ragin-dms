use std::fs;
use std::path::Path;
use std::sync::Arc;

use dms_application::port::storage::{CatalogLoad, CatalogRepository, StorageError, StorageFailure, StorageResult};
use dms_domain::catalog::{Catalog, CatalogRevision};
use dms_domain::library::LibraryFolder;

use crate::catalog::dto::CatalogDto;
use crate::catalog::mapper::catalog_mapper;
use crate::catalog::{CatalogFiles, CatalogMigrations};
use crate::fs::{OrIoError, atomic_files, failure, root_of};
use crate::platform::SystemTrash;
use crate::yaml::{emit, parse_bytes};

/// Reads and writes `dms-catalog.yaml` in the root of one library folder.
///
/// A file this version cannot read, because it is newer or broken, is never overwritten: saving fails instead, so
/// no device loses what another one wrote. The same holds for a file with a higher revision than the change was
/// based on.
pub struct FileCatalogRepository {
    trash: Arc<dyn SystemTrash>,
}

impl FileCatalogRepository {
    pub fn new(trash: Arc<dyn SystemTrash>) -> Self {
        Self { trash }
    }

    fn load_found(root: &Path) -> StorageResult<CatalogLoad> {
        let file = root.join(CatalogFiles::FILE_NAME);
        if !file.exists() {
            return Ok(CatalogLoad::NotFound);
        }
        let catalog = read_catalog(&file)?;
        let conflict_copies =
            CatalogFiles::conflict_copies(root).or_io()?.iter().filter_map(|copy| read_catalog(copy).ok()).collect();
        Ok(CatalogLoad::Found { catalog, conflict_copies })
    }
}

impl CatalogRepository for FileCatalogRepository {
    fn load(&self, folder: &LibraryFolder) -> CatalogLoad {
        root_of(folder).and_then(|root| Self::load_found(&root)).unwrap_or_else(CatalogLoad::Failed)
    }

    fn save(&self, folder: &LibraryFolder, catalog: &Catalog, based_on: CatalogRevision) -> StorageResult<()> {
        let root = root_of(folder)?;
        let file = root.join(CatalogFiles::FILE_NAME);
        let previous = if file.exists() { Some(read_dto(&file)?) } else { None };
        if previous.as_ref().is_some_and(|previous| i128::from(previous.revision) > i128::from(based_on.value())) {
            return Err(failure(StorageFailure::CatalogChanged));
        }
        let dto = catalog_mapper::to_dto(catalog, previous.as_ref());
        atomic_files::write(&file, emit(&dto.write()).as_bytes()).or_io()
    }

    /// Moves the conflict copies to the trash. Copies that cannot be read stay, so nothing unread is lost.
    fn discard_conflict_copies(&self, folder: &LibraryFolder) -> StorageResult<()> {
        let root = root_of(folder)?;
        let readable: Vec<_> = CatalogFiles::conflict_copies(&root)
            .or_io()?
            .into_iter()
            .filter(|copy| read_catalog(copy).is_ok())
            .collect();
        let failed = readable.iter().filter(|copy| !self.trash.move_to_trash(&root, copy)).count();
        if failed == 0 { Ok(()) } else { Err(StorageError::with_detail(StorageFailure::IoError, "Trash failed")) }
    }
}

fn read_catalog(file: &Path) -> StorageResult<Catalog> {
    Ok(catalog_mapper::to_domain(&read_dto(file)?)?)
}

fn read_dto(file: &Path) -> StorageResult<CatalogDto> {
    let bytes = fs::read(file).or_io()?;
    let tree = parse_bytes(&bytes)?.ok_or_else(|| failure(StorageFailure::UnsupportedFormat))?;
    let migrated = CatalogMigrations::migrate(tree).ok_or_else(|| failure(StorageFailure::UnsupportedFormat))?;
    Ok(CatalogDto::read(&migrated)?)
}
