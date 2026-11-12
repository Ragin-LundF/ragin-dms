/// How one part of the catalog (documents, categories, …) would change: entries restoring brings back
/// (`added`), drops (`removed`) and changes (`changed`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PartDiff {
    pub added: usize,
    pub removed: usize,
    pub changed: usize,
}

impl PartDiff {
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.added == 0 && self.removed == 0 && self.changed == 0
    }
}
