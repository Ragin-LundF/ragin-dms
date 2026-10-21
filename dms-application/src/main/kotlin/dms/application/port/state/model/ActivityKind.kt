package dms.application.port.state.model

/** What the tool did, for the dashboard's recent activity. */
enum class ActivityKind {
    DOCUMENT_ADDED,
    DOCUMENT_MOVED,
    DOCUMENT_TRASHED,
    DOCUMENT_RESTORED,
    FOLDER_CREATED,
    FOLDER_MOVED,
    CATALOG_EDITED,
    SUGGESTION_APPROVED,
    CONFLICT_RESOLVED,
    SYNCHRONIZED,
}
