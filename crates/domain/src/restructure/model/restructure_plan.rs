use indexmap::IndexSet;

use crate::document::DocumentId;
use crate::library::RelativePath;
use crate::restructure::{PlannedMove, SkippedDocument};

/// What a restructure would do: the [`Self::moves`], the documents already in place ([`Self::unchanged`]) and
/// those left where they are ([`Self::skipped`]). [`Self::renamed_for_clash`] are the moves whose name got a ` (2)`
/// suffix because another document already has that name in the target folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestructurePlan {
    pub template: String,
    pub moves: Vec<PlannedMove>,
    pub unchanged: Vec<DocumentId>,
    pub skipped: Vec<SkippedDocument>,
    pub renamed_for_clash: IndexSet<DocumentId>,
}

impl RestructurePlan {
    /// The folders the documents leave, which may be empty afterwards.
    #[must_use]
    pub fn source_folders(&self) -> IndexSet<RelativePath> {
        self.moves.iter().filter_map(|planned| planned.from.parent()).filter(|folder| !folder.is_root()).collect()
    }

    /// This plan with only the moves `keep` accepts, for example after the user unticked a subtree.
    #[must_use]
    pub fn only(&self, keep: impl Fn(&PlannedMove) -> bool) -> Self {
        let kept: Vec<PlannedMove> = self.moves.iter().filter(|planned| keep(planned)).cloned().collect();
        let renamed_for_clash = self
            .renamed_for_clash
            .iter()
            .filter(|id| kept.iter().any(|planned| &planned.document_id == *id))
            .cloned()
            .collect();
        Self { moves: kept, renamed_for_clash, ..self.clone() }
    }
}
