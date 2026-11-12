use std::path::PathBuf;

use dms_application::port::health::{FolderCheck, HealthReport, HealthStore};
use dms_application::port::storage::StorageResult;
use dms_domain::catalog::LibraryId;
use dms_domain::document::DocumentId;
use dms_domain::health::HealthFinding;
use dms_domain::library::LibraryFolderId;

use crate::catalog::dto::read_list;
use crate::instant;
use crate::yaml::{Fields, MapWriter, Node, YamlError, YamlFile};

/// `health.yaml` in the app data folder: the report of the last archive check (new in 2.1, so no 1.x format to
/// keep). Instants as in every other file, ids as text, findings with a `kind`.
pub struct FileHealthStore {
    file: YamlFile,
}

impl FileHealthStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { file: YamlFile::new(path) }
    }
}

impl HealthStore for FileHealthStore {
    fn load(&self) -> StorageResult<Option<HealthReport>> {
        Ok(self.file.read()?.map(|node| read_report(&node)).transpose()?)
    }

    fn save(&self, report: &HealthReport) -> StorageResult<()> {
        self.file.write(&write_report(report))
    }
}

fn read_report(node: &Node) -> Result<HealthReport, YamlError> {
    let fields = Fields::of(node)?;
    let instant_of = |key: &str| fields.opt_text(key)?.map(instant::parse).transpose();
    Ok(HealthReport {
        library_id: LibraryId::new(fields.text("libraryId")?)?,
        started_at: instant_of("startedAt")?,
        finished_at: instant_of("finishedAt")?,
        resume_after: fields.opt_text("resumeAfter")?.map(DocumentId::new).transpose()?,
        folders: read_list(fields, "folders", read_folder)?,
        findings: read_list(fields, "findings", read_finding)?,
    })
}

fn read_folder(node: &Node) -> Result<FolderCheck, YamlError> {
    let fields = Fields::of(node)?;
    let count =
        |key: &str| fields.opt_i64(key).map(|value| u32::try_from(value.unwrap_or_default()).unwrap_or_default());
    Ok(FolderCheck {
        folder_id: LibraryFolderId::new(fields.text("folderId")?)?,
        checked: count("checked")?,
        not_downloaded: count("notDownloaded")?,
        missing: count("missing")?,
        offline: fields.bool_or("offline", false)?,
    })
}

fn read_finding(node: &Node) -> Result<HealthFinding, YamlError> {
    let fields = Fields::of(node)?;
    let document_id = DocumentId::new(fields.text("documentId")?)?;
    match fields.text("kind")? {
        "damaged" => Ok(HealthFinding::Damaged {
            document_id,
            folder_id: LibraryFolderId::new(fields.text("folderId")?)?,
            source_folder_id: LibraryFolderId::new(fields.text("sourceFolderId")?)?,
        }),
        "unreadable" => Ok(HealthFinding::Unreadable { document_id }),
        other => Err(YamlError(format!("Unknown finding: {other}"))),
    }
}

fn write_report(report: &HealthReport) -> Node {
    MapWriter::new()
        .text("libraryId", report.library_id.value())
        .opt_text("startedAt", report.started_at.map(instant::format).as_deref())
        .opt_text("finishedAt", report.finished_at.map(instant::format).as_deref())
        .opt_text("resumeAfter", report.resume_after.as_ref().map(DocumentId::value))
        .node("folders", Node::Seq(report.folders.iter().map(write_folder).collect()))
        .node("findings", Node::Seq(report.findings.iter().map(write_finding).collect()))
        .build()
}

fn write_folder(check: &FolderCheck) -> Node {
    MapWriter::new()
        .text("folderId", check.folder_id.value())
        .int("checked", check.checked)
        .int("notDownloaded", check.not_downloaded)
        .int("missing", check.missing)
        .bool("offline", check.offline)
        .build()
}

fn write_finding(finding: &HealthFinding) -> Node {
    match finding {
        HealthFinding::Damaged { document_id, folder_id, source_folder_id } => MapWriter::new()
            .text("kind", "damaged")
            .text("documentId", document_id.value())
            .text("folderId", folder_id.value())
            .text("sourceFolderId", source_folder_id.value()),
        HealthFinding::Unreadable { document_id } => {
            MapWriter::new().text("kind", "unreadable").text("documentId", document_id.value())
        }
    }
    .build()
}
