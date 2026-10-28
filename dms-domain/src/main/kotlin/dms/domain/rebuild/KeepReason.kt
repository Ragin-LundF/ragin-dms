package dms.domain.rebuild

/** Why a text line of a rebuilt page stayed pixels of the scan instead of becoming real text. */
enum class KeepReason {
    /** The AI was not sure about every character. */
    NOT_SURE,

    /** No ink was found where the AI placed the line. */
    NO_INK,

    /** A stamp, signature or handwriting covers the line. */
    COVERED,

    /** The font has no glyph for a character of the line. */
    NOT_ENCODABLE,

    /** The text is wider or narrower than the ink: words are missing or invented. */
    TEXT_MISMATCH,

    /** The AI lost characters, and reading the line again did not recover them. */
    LOST_CHARACTERS,

    /** The AI could not read the page, so the whole page stayed as it was. */
    PAGE_FAILED,
}
