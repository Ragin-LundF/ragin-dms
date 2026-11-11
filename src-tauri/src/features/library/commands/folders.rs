//! Creating, renaming, moving and deleting folders in every library folder (Kotlin `FolderEditor`), and adding
//! files to a folder (Kotlin `FileAdder`). The folder changes wait for the operation and return whether it was done,
//! so the folder view follows a moved or deleted folder; refusals and a busy library are reported as messages.

use dms_application::common::OperationOutcome;
use dms_application::document::AddDocumentsOutcome;
use dms_application::folder::FolderEntriesOutcome;
use dms_application::port::storage::ExternalFile;
use dms_application::services::AppServices;
use dms_domain::library::{FileNamePolicy, RelativePath};
use dms_domain::text;

use super::operation::{start, wait};
use super::view::parse_paths;
use crate::commands::AppState;
use crate::dto::IpcError;
use crate::features::library::dto::folders::LibraryConfirmationDto;
use crate::features::library::dto::params::parse_path;
use crate::features::library::view::folders::delete_confirmation;
use crate::messages;
use crate::session::reporting::blocking;
use crate::session::{Lane, UiMessage};

/// Creates the folder `name` in `parent`, as typed in place.
#[tauri::command]
#[specta::specta]
pub async fn library_create_folder(ctx: AppState<'_>, parent: String, name: String) -> Result<bool, IpcError> {
    let parent = parse_path(&parent)?;
    let Some(target) = typed_child(&ctx, &parent, &name, None)? else {
        return Ok(false);
    };
    let done = format!("Created \"{}\"", target.file_name());
    run(&ctx, format!("Creating folder {}", target.value()), done, move |services| {
        services.documents.create_folder.run(&target)
    })
    .await
}

/// Renames the folder at `path` to `name`, as typed in place; an unchanged or blank name changes nothing.
#[tauri::command]
#[specta::specta]
pub async fn library_rename_folder(ctx: AppState<'_>, path: String, name: String) -> Result<bool, IpcError> {
    let path = parse_path(&path)?;
    if path.is_root() {
        ctx.session.report(UiMessage::error("The library's top level cannot be renamed"));
        return Ok(false);
    }
    let parent = path.parent().unwrap_or_else(RelativePath::root);
    let Some(target) = typed_child(&ctx, &parent, &name, Some(path.file_name()))? else {
        return Ok(false);
    };
    let done = format!("Renamed to \"{}\" in every library folder", target.file_name());
    let label = format!("Moving folder {}", path.value());
    run(&ctx, label, done, move |services| services.documents.move_folder.run(&path, &target)).await
}

/// Moves the folder at `path` into `folder`; into itself or a subfolder is refused, where it already is does nothing.
#[tauri::command]
#[specta::specta]
pub async fn library_move_folder_into(ctx: AppState<'_>, path: String, folder: String) -> Result<bool, IpcError> {
    let path = parse_path(&path)?;
    let folder = parse_path(&folder)?;
    if folder == path || folder.is_inside(&path) {
        ctx.session.report(UiMessage::error("A folder cannot be moved into itself or one of its subfolders"));
        return Ok(false);
    }
    let target = folder.resolve(path.file_name()).map_err(|error| IpcError::invalid(error.to_string()))?;
    if target == path {
        return Ok(false);
    }
    let where_ = if folder.is_root() { "the top level" } else { folder.value() };
    let done = format!("Moved \"{}\" to {where_}", path.file_name());
    let label = format!("Moving folder {}", path.value());
    run(&ctx, label, done, move |services| services.documents.move_folder.run(&path, &target)).await
}

/// The question before deleting `path`, naming the documents, the files the library does not know and the
/// subfolders that go along; `null` (after a message) when the folder cannot be deleted now.
#[tauri::command]
#[specta::specta]
pub async fn library_folder_delete_request(
    ctx: AppState<'_>,
    path: String,
    folders_on_disk: Vec<String>,
) -> Result<Option<LibraryConfirmationDto>, IpcError> {
    let path = parse_path(&path)?;
    let on_disk = parse_paths(&folders_on_disk)?;
    if path.is_root() {
        ctx.session.report(UiMessage::error("The library's top level cannot be deleted"));
        return Ok(None);
    }
    let ctx = ctx.inner().clone();
    blocking(move || {
        // Without knowing what is inside, the question could not name it: nothing is offered then.
        match ctx.require_services()?.documents.read_folder_entries.run(&path) {
            FolderEntriesOutcome::Read(entries) => {
                let catalog = ctx.require_catalog()?;
                let files: Vec<RelativePath> = entries.files.into_iter().collect();
                let folders: Vec<RelativePath> = entries.folders.into_iter().collect();
                Ok(Some(delete_confirmation(&catalog, &path, &files, &folders, &on_disk)))
            }
            FolderEntriesOutcome::LibraryNotOpen => {
                ctx.session.report(messages::of(&OperationOutcome::LibraryNotOpen, ""));
                Ok(None)
            }
            FolderEntriesOutcome::StorageFailed(error) => {
                ctx.session.report(UiMessage::error(format!(
                    "\"{}\" cannot be deleted now: its contents could not be read ({})",
                    path.file_name(),
                    messages::failure(error.reason)
                )));
                Ok(None)
            }
        }
    })
    .await
}

/// Trashes the folder at `path` in every library folder; the screen asked first, listing `listedFiles` files.
#[tauri::command]
#[specta::specta]
pub async fn library_trash_folder(ctx: AppState<'_>, path: String, listed_files: u32) -> Result<bool, IpcError> {
    let path = parse_path(&path)?;
    if path.is_root() {
        ctx.session.report(UiMessage::error("The library's top level cannot be deleted"));
        return Ok(false);
    }
    let done = if listed_files == 0 {
        format!("Deleted \"{}\"", path.file_name())
    } else {
        format!("Deleted \"{}\"; its documents are in the trash", path.file_name())
    };
    let label = format!("Deleting folder {}", path.value());
    run(&ctx, label, done, move |services| services.documents.trash_folder.run(&path, true)).await
}

/// Copies the files at `paths` (absolute) into `target` in every library folder. Folders among them, as a drop
/// from the file manager can carry, are left out; a busy library does nothing.
#[tauri::command]
#[specta::specta]
pub async fn library_add_files(ctx: AppState<'_>, paths: Vec<String>, target: String) -> Result<(), IpcError> {
    let target = parse_path(&target)?;
    let files: Vec<ExternalFile> = paths
        .into_iter()
        .filter(|path| std::path::Path::new(path).is_file())
        .map(|path| ExternalFile::new(path).map_err(|error| IpcError::invalid(error.to_string())))
        .collect::<Result<_, _>>()?;
    if files.is_empty() {
        ctx.session.report(UiMessage::error("Only files can be added, not folders"));
        return Ok(());
    }
    let label = format!("Adding {} file(s)", files.len());
    let started = start(ctx.inner(), &label, Lane::Library, false, move |services, operation| {
        let message = match services.documents.add_documents.run(&files, &target) {
            AddDocumentsOutcome::Added(added) => {
                let done = added.iter().filter(|file| matches!(file.outcome, OperationOutcome::Done { .. })).count();
                let duplicates = added.iter().filter(|file| !file.duplicates.is_empty()).count();
                let mut parts = vec![format!("Added {done} of {} file(s)", files.len())];
                if duplicates > 0 {
                    parts.push(format!("{duplicates} already in the library under another name"));
                }
                if let Some(problem) = added.iter().find(|file| !matches!(file.outcome, OperationOutcome::Done { .. }))
                {
                    parts.push(format!("{}: {}", problem.source.file_name(), messages::of(&problem.outcome, "").text));
                }
                let text = parts.join("; ");
                if done < files.len() { UiMessage::error(text) } else { UiMessage::info(text) }
            }
            AddDocumentsOutcome::LibraryNotOpen => messages::of(&OperationOutcome::LibraryNotOpen, ""),
        };
        operation.session().report(message);
    });
    match started {
        Err(error) if error.code != "busy" => Err(error),
        _ => Ok(()),
    }
}

/// Runs a folder change as a library operation, reports its outcome with `done`, and returns whether it was done;
/// a busy library is said in a message.
async fn run(
    ctx: &AppState<'_>,
    label: String,
    done: String,
    job: impl FnOnce(&AppServices) -> OperationOutcome + Send + 'static,
) -> Result<bool, IpcError> {
    let result = wait(ctx.inner(), &label, move |services, operation| {
        let outcome = job(services);
        operation.session().report(messages::of(&outcome, &done));
        matches!(outcome, OperationOutcome::Done { .. })
    })
    .await;
    match result {
        Err(error) if error.code == "busy" => {
            ctx.session.report(UiMessage::error(error.message));
            Ok(false)
        }
        other => other,
    }
}

/// The folder `parent`/`name` for a typed name: trimmed, `/` turned into `-`. `None` when nothing is to be done (a
/// blank name, or `unchanged`), or after reporting why the name cannot be one.
fn typed_child(
    ctx: &AppState<'_>,
    parent: &RelativePath,
    raw: &str,
    unchanged: Option<&str>,
) -> Result<Option<RelativePath>, IpcError> {
    let name = text::trim(raw).replace('/', "-");
    if name.is_empty() || Some(name.as_str()) == unchanged {
        return Ok(None);
    }
    if let Some(violation) = FileNamePolicy::check(&name) {
        ctx.session.report(UiMessage::error(messages::name_problem(violation)));
        return Ok(None);
    }
    parent.resolve(&name).map(Some).map_err(|error| IpcError::invalid(error.to_string()))
}
