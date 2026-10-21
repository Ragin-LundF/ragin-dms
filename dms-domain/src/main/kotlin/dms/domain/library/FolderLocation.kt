package dms.domain.library

/**
 * Where a library folder lives on this device, as an absolute path in the device's own syntax.
 *
 * Locations differ per device and are therefore part of the device settings, never of the catalog.
 */
@JvmInline
value class FolderLocation(val value: String) {
    init {
        require(value = value.isNotBlank()) { "Folder location must not be blank" }
    }

    override fun toString(): String {
        return value
    }
}
