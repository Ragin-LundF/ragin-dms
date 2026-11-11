use std::sync::Arc;

use crate::catalog::{ChangeSettings, EditCatalog};
use crate::deadline::SettleRetention;
use crate::document::{
    AddDocuments, AdoptNewFile, EditDocumentDetails, MoveDocument, RestoreDocument, TrashDocument, TrashDuplicateFile,
    TrashNewFile,
};
use crate::folder::{CreateFolder, MoveFolder, ReadFolderEntries, ReadFolders, TrashFolder};

/// Use cases that change documents, folders, their categorization and details, and the library settings.
pub struct DocumentServices {
    pub add_documents: Arc<AddDocuments>,
    pub adopt_new_file: Arc<AdoptNewFile>,
    pub move_document: Arc<MoveDocument>,
    pub trash_document: Arc<TrashDocument>,
    pub trash_new_file: Arc<TrashNewFile>,
    pub trash_duplicate_file: Arc<TrashDuplicateFile>,
    pub restore_document: Arc<RestoreDocument>,
    pub create_folder: Arc<CreateFolder>,
    pub move_folder: Arc<MoveFolder>,
    pub read_folder_entries: Arc<ReadFolderEntries>,
    pub read_folders: Arc<ReadFolders>,
    pub trash_folder: Arc<TrashFolder>,
    pub edit_catalog: Arc<EditCatalog>,
    pub change_settings: Arc<ChangeSettings>,
    pub edit_document_details: Arc<EditDocumentDetails>,
    pub settle_retention: Arc<SettleRetention>,
}
