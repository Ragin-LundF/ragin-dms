package dms.domain.library

/**
 * The primary folder is written first and is the source for "sync to all destinations".
 * Sync folders hold the same library and receive every change made in the tool.
 */
enum class FolderRole {
    PRIMARY,
    SYNC,
}
