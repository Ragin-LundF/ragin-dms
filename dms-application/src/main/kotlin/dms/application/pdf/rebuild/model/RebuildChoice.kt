package dms.application.pdf.rebuild.model

/** What the user does with a rebuilt PDF. */
enum class RebuildChoice {
    KEEP_ORIGINAL,
    REPLACE_WITH_REBUILD,
    KEEP_BOTH,
}
