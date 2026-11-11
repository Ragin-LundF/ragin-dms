/// Why a file or folder name is not allowed for a new entry. See [`crate::library::FileNamePolicy`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileNameViolation {
    Empty,
    TooLong,
    ControlCharacter,
    ForbiddenCharacter,
    ReservedName,
    LeadingDotOrSpace,
    TrailingDotOrSpace,
}
