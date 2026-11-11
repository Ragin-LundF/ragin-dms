/// Who produced the categorization the user confirmed last: the AI, the user, or the matching rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClassificationSource {
    Ai,
    Manual,
    Rule,
}
