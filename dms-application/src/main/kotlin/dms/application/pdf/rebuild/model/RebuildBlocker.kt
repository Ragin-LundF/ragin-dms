package dms.application.pdf.rebuild.model

/** Why a document cannot be rebuilt. */
enum class RebuildBlocker {
    NOT_PDF,
    HAS_TEXT_LAYER,

    /** The AI server cannot be reached or is not set up. */
    AI_NOT_REACHABLE,
    MODEL_CANNOT_READ_IMAGES,

    /** The PDF changed while it was converted, so the pages would come from two versions. */
    SOURCE_CHANGED,
}
