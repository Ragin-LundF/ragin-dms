package dms.application.resolution.model

/** What to do about a file that was moved outside the tool. */
enum class MoveDecision {
    /** Keep the new path: move the file in the other folders too and update the catalog. */
    APPLY,

    /** Move the file back to the path the catalog knows. */
    UNDO,
}
