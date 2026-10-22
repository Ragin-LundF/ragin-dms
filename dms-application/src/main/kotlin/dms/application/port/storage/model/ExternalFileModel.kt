package dms.application.port.storage.model

/**
 * A file outside the library that the user wants to add, for example one dropped from the file
 * manager. [location] is an absolute path in the syntax of this device.
 */
@JvmInline
value class ExternalFileModel(val location: String) {
    init {
        require(value = location.isNotBlank()) { "Location must not be blank" }
    }

    /** The last path segment, for both `/` and `\` separated locations. */
    val fileName: String
        get() {
            return location.trimEnd('/', '\\').substringAfterLast(delimiter = '/').substringAfterLast(delimiter = '\\')
        }

    override fun toString(): String {
        return location
    }
}
