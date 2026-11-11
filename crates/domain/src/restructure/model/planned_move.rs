use crate::document::DocumentId;
use crate::library::RelativePath;

/// One document of a restructure, moving [`Self::from`] its current path [`Self::to`] its new one.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlannedMove {
    pub document_id: DocumentId,
    pub from: RelativePath,
    pub to: RelativePath,
}

impl PlannedMove {
    /// The move that undoes this one.
    #[must_use]
    pub fn inverse(&self) -> Self {
        Self { document_id: self.document_id.clone(), from: self.to.clone(), to: self.from.clone() }
    }
}
