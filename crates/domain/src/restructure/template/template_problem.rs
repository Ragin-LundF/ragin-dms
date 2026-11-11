/// Why a text is no valid [`crate::restructure::PathTemplate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TemplateProblem {
    Empty,
    UnclosedBrace,
    UnknownPlaceholder,
    InvalidFieldKey,
    /// A fixed folder name breaks the file name rules, such as `..` or a name with `:`.
    InvalidFolderName,
}
