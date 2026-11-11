use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use dms_application::port::plan::{PlanStore, RestructureRecord};
use dms_application::port::storage::{StorageError, StorageFailure, StorageResult};
use dms_domain::document::DocumentId;
use dms_domain::library::{LibraryFolder, RelativePath};
use dms_domain::restructure::PlannedMove;
use dms_domain::text;

use crate::fs::{OrIoError, atomic_files, path_resolver, root_of};
use crate::instant;
use crate::yaml::{Fields, MapWriter, Node, YamlError, emit, parse_bytes};

/// Applied restructures in `.dms-plans/<name>.yaml` of one library folder: `appliedAt`, `template` and the
/// `moves` (`documentId`, `from`, `to`). A broken file reads as none.
#[derive(Debug, Clone, Copy, Default)]
pub struct FilePlanStore;

const EXTENSION: &str = ".yaml";

impl FilePlanStore {
    pub const PLANS_FOLDER: &'static str = ".dms-plans";

    /// `None` for a name that is not `^[A-Za-z0-9_-]+$`, so a name never leads outside the folder.
    fn file_of(root: &Path, name: &str) -> Option<PathBuf> {
        let valid =
            !name.is_empty() && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'));
        valid.then(|| root.join(Self::PLANS_FOLDER).join(format!("{name}{EXTENSION}")))
    }
}

impl PlanStore for FilePlanStore {
    fn write(&self, folder: &LibraryFolder, record: &RestructureRecord) -> StorageResult<()> {
        let root = root_of(folder)?;
        // The Kotlin version threw for such a name; the use cases only make valid ones.
        let file = Self::file_of(&root, &record.name).ok_or_else(|| {
            StorageError::with_detail(StorageFailure::IoError, format!("Not a plan name: {}", record.name))
        })?;
        atomic_files::write(&file, emit(&to_node(record)).as_bytes()).or_io()
    }

    fn read(&self, folder: &LibraryFolder, name: &str) -> Option<RestructureRecord> {
        let file = Self::file_of(&path_resolver::root(folder)?, name)?;
        let bytes = file.is_file().then(|| fs::read(file).ok()).flatten()?;
        let node = parse_bytes(&bytes).ok()??;
        to_record(name, &node).ok()
    }

    fn names(&self, folder: &LibraryFolder) -> Vec<String> {
        let Some(directory) = path_resolver::root(folder).map(|root| root.join(Self::PLANS_FOLDER)) else {
            return Vec::new();
        };
        let Ok(entries) = fs::read_dir(directory) else { return Vec::new() };
        let mut names: Vec<String> = entries
            .filter_map(Result::ok)
            .filter_map(|entry| entry.file_name().to_string_lossy().strip_suffix(EXTENSION).map(str::to_owned))
            .collect();
        names.sort_by(|first, second| text::compare(second, first));
        names
    }
}

fn to_node(record: &RestructureRecord) -> Node {
    let moves = record
        .moves
        .iter()
        .map(|planned| {
            MapWriter::new()
                .text("documentId", planned.document_id.value())
                .text("from", planned.from.value())
                .text("to", planned.to.value())
                .build()
        })
        .collect();
    MapWriter::new()
        .text("appliedAt", &instant::format(record.applied_at))
        .text("template", &record.template)
        .node("moves", Node::Seq(moves))
        .build()
}

fn to_record(name: &str, node: &Node) -> Result<RestructureRecord, YamlError> {
    let fields = Fields::of(node)?;
    let applied_at: DateTime<Utc> = instant::parse(fields.text("appliedAt")?)?;
    let moves = fields
        .list("moves")?
        .iter()
        .map(|item| {
            let planned = Fields::of(item)?;
            Ok(PlannedMove {
                document_id: DocumentId::new(planned.text("documentId")?).map_err(|error| invalid(&error))?,
                from: RelativePath::of(planned.text("from")?).map_err(|error| invalid(&error))?,
                to: RelativePath::of(planned.text("to")?).map_err(|error| invalid(&error))?,
            })
        })
        .collect::<Result<_, YamlError>>()?;
    Ok(RestructureRecord { name: name.to_owned(), applied_at, template: fields.text("template")?.to_owned(), moves })
}

fn invalid(error: &dms_domain::DomainError) -> YamlError {
    YamlError(error.to_string())
}
