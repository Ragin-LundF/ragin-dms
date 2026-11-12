use crate::document::Document;
use crate::health::{CopyObservation, CopyState, HealthFinding};
use crate::library::LibraryFolderId;

/// Turns what a check saw of one document's copies into findings.
///
/// - Copies disagree and at least one matches the catalog: every other read copy is damaged, repaired from the
///   primary when it matches, else from the first matching folder.
/// - All read copies agree but differ from the catalog: no finding; that is a modification the scan reports.
/// - A copy that cannot be read counts as damaged.
/// - A PDF that opens in no read copy, or a file no folder can read: unreadable.
/// - Missing and not downloaded copies are no finding; the scan's sync tasks deal with missing files.
pub struct HealthJudge;

impl HealthJudge {
    #[must_use]
    pub fn judge(document: &Document, copies: &[CopyObservation], primary: &LibraryFolderId) -> Vec<HealthFinding> {
        let read: Vec<(&LibraryFolderId, bool, Option<bool>)> = copies
            .iter()
            .filter_map(|copy| match &copy.state {
                CopyState::Read { content_hash, size, opens } => {
                    Some((&copy.folder_id, *content_hash == document.content_hash && *size == document.size, *opens))
                }
                CopyState::ReadFailed => Some((&copy.folder_id, false, Some(false))),
                CopyState::Missing | CopyState::NotDownloaded => None,
            })
            .collect();
        let mut findings = Vec::new();
        let matching: Vec<&LibraryFolderId> =
            read.iter().filter(|(_, matches, _)| *matches).map(|(folder, _, _)| *folder).collect();
        let source = matching.iter().find(|folder| **folder == primary).or_else(|| matching.first());
        if let Some(source) = source {
            findings.extend(read.iter().filter(|(_, matches, _)| !matches).map(|(folder, _, _)| {
                HealthFinding::Damaged {
                    document_id: document.id.clone(),
                    folder_id: (*folder).clone(),
                    source_folder_id: (*source).clone(),
                }
            }));
        }
        if !read.is_empty() && read.iter().all(|(_, _, opens)| *opens == Some(false)) {
            findings.push(HealthFinding::Unreadable { document_id: document.id.clone() });
        }
        findings
    }
}
