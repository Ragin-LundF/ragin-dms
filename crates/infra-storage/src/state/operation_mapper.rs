//! Journal entries in `journal.yaml`.

use dms_application::port::storage::{FolderOperation, PendingOperation};
use dms_domain::document::DocumentId;
use dms_domain::library::{LibraryFolderId, RelativePath};

use crate::catalog::dto::owned;
use crate::instant;
use crate::yaml::{Fields, MapWriter, Node, YamlError};

const COPY: &str = "copy";
const CREATE_FOLDER: &str = "createFolder";
const MOVE: &str = "move";
const TRASH: &str = "trash";
const COPY_TEXT: &str = "copyText";
const SAVE_CATALOG: &str = "saveCatalog";
const COPY_PLAN: &str = "copyPlan";
const COPY_INDEX: &str = "copyIndex";

/// One journal entry; `type` names the operation and only the fields it needs are set. `NON_NULL`.
#[derive(Debug, Default)]
struct PendingOperationDto {
    id: String,
    folder_id: String,
    recorded_at: String,
    operation_type: String,
    path: Option<String>,
    from: Option<String>,
    to: Option<String>,
    replace_existing: Option<bool>,
    document_id: Option<String>,
}

pub fn to_node(operation: &PendingOperation) -> Node {
    let mut dto = PendingOperationDto {
        id: operation.id.clone(),
        folder_id: operation.folder_id.value().to_owned(),
        recorded_at: instant::format(operation.recorded_at),
        operation_type: SAVE_CATALOG.to_owned(),
        ..PendingOperationDto::default()
    };
    let (operation_type, path) = match &operation.operation {
        FolderOperation::CopyFromPrimary { path, replace_existing } => {
            dto.replace_existing = Some(*replace_existing);
            (COPY, Some(path.value()))
        }
        FolderOperation::CreateFolder(path) => (CREATE_FOLDER, Some(path.value())),
        FolderOperation::Move { from, to } => {
            dto.from = Some(from.value().to_owned());
            dto.to = Some(to.value().to_owned());
            (MOVE, None)
        }
        FolderOperation::MoveToTrash(path) => (TRASH, Some(path.value())),
        FolderOperation::CopyText(document_id) => {
            dto.document_id = Some(document_id.value().to_owned());
            (COPY_TEXT, None)
        }
        FolderOperation::CopyPlan(name) => (COPY_PLAN, Some(name.as_str())),
        FolderOperation::CopyIndex(document_id) => {
            dto.document_id = Some(document_id.value().to_owned());
            (COPY_INDEX, None)
        }
        FolderOperation::SaveCatalog => (SAVE_CATALOG, None),
    };
    operation_type.clone_into(&mut dto.operation_type);
    dto.path = owned(path);
    MapWriter::new()
        .text("id", &dto.id)
        .text("folderId", &dto.folder_id)
        .text("recordedAt", &dto.recorded_at)
        .text("type", &dto.operation_type)
        .opt_text("path", dto.path.as_deref())
        .opt_text("from", dto.from.as_deref())
        .opt_text("to", dto.to.as_deref())
        .opt_node("replaceExisting", dto.replace_existing.map(Node::Bool))
        .opt_text("documentId", dto.document_id.as_deref())
        .build()
}

/// The journal entry, or `None` for an entry this version does not understand.
///
/// # Errors
/// Fails for an entry without its required fields or with an invalid value, as the Kotlin version threw.
pub fn to_domain(node: &Node) -> Result<Option<PendingOperation>, YamlError> {
    let fields = Fields::of(node)?;
    let dto = PendingOperationDto {
        id: fields.text("id")?.to_owned(),
        folder_id: fields.text("folderId")?.to_owned(),
        recorded_at: fields.text("recordedAt")?.to_owned(),
        operation_type: fields.text("type")?.to_owned(),
        path: owned(fields.opt_text("path")?),
        from: owned(fields.opt_text("from")?),
        to: owned(fields.opt_text("to")?),
        replace_existing: fields.opt_bool("replaceExisting")?,
        document_id: owned(fields.opt_text("documentId")?),
    };
    let Some(operation) = operation_of(&dto)? else { return Ok(None) };
    Ok(Some(PendingOperation {
        id: dto.id,
        folder_id: LibraryFolderId::new(dto.folder_id)?,
        operation,
        recorded_at: instant::parse(&dto.recorded_at)?,
    }))
}

fn operation_of(dto: &PendingOperationDto) -> Result<Option<FolderOperation>, YamlError> {
    let path = || dto.path.as_deref().map(RelativePath::of).transpose();
    let document_id = || dto.document_id.as_deref().map(DocumentId::new).transpose();
    Ok(match dto.operation_type.as_str() {
        COPY => path()?.map(|path| FolderOperation::CopyFromPrimary {
            path,
            replace_existing: dto.replace_existing == Some(true),
        }),
        CREATE_FOLDER => path()?.map(FolderOperation::CreateFolder),
        TRASH => path()?.map(FolderOperation::MoveToTrash),
        MOVE => match (&dto.from, &dto.to) {
            (Some(from), Some(to)) => {
                Some(FolderOperation::Move { from: RelativePath::of(from)?, to: RelativePath::of(to)? })
            }
            _ => None,
        },
        COPY_TEXT => document_id()?.map(FolderOperation::CopyText),
        COPY_INDEX => document_id()?.map(FolderOperation::CopyIndex),
        COPY_PLAN => dto.path.clone().map(FolderOperation::CopyPlan),
        SAVE_CATALOG => Some(FolderOperation::SaveCatalog),
        _ => None,
    })
}
