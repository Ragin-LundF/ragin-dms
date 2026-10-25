package dms.application.port.storage.model

/** Why a file system operation in a library folder failed. */
enum class StorageFailure {
    /** The folder is not reachable right now, for example an offline network share. */
    FOLDER_UNREACHABLE,
    ALREADY_EXISTS,
    NOT_FOUND,

    /** The copy was written but its content hash does not match the source. */
    VERIFICATION_FAILED,
    IO_ERROR,

    /** A file written by a newer version of the tool, or not a file this tool can read. */
    UNSUPPORTED_FORMAT,

    /** The catalog file holds a newer revision than the change was based on; nothing was written. */
    CATALOG_CHANGED,
}
