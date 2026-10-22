package dms.domain.restructure

/** Why a restructure leaves a document where it is. */
enum class SkipReason {
    MISSING_CATEGORY,
    MISSING_DATE,
    MISSING_CORRESPONDENT,
    MISSING_FIELD,

    /** Inbox documents are filed one by one, never restructured. */
    IN_INBOX,
}
