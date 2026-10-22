package dms.domain.restructure

/** What a restructure does with file names. */
enum class FileNaming {
    /** Keep every file name as it is. */
    KEEP,

    /** `2024-03-15 Title.pdf` where the document has a date and a title; the name as it is otherwise. */
    DATE_AND_TITLE,
}
