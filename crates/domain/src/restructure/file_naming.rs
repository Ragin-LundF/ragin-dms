/// What a restructure does with file names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileNaming {
    /// Keep every file name as it is.
    Keep,
    /// `2024-03-15 Title.pdf` where the document has a date and a title; the name as it is otherwise.
    DateAndTitle,
}
