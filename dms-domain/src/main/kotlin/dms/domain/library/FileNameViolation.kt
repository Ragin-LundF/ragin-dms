package dms.domain.library

/** Why a file or folder name is not allowed for a new entry. See [FileNamePolicy]. */
enum class FileNameViolation {
    EMPTY,
    TOO_LONG,
    CONTROL_CHARACTER,
    FORBIDDEN_CHARACTER,
    RESERVED_NAME,
    LEADING_DOT_OR_SPACE,
    TRAILING_DOT_OR_SPACE,
}
