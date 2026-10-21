package dms.storage.scan

/**
 * Files a sync client keeps only in the cloud. Reading one downloads it, so a scan reports it under
 * its real name without reading it.
 */
object CloudPlaceholders {

    private const val ICLOUD_SUFFIX = ".icloud"

    /** iCloud Drive stores a file that is not downloaded as `.<name>.icloud`; returns `<name>`, or `null`. */
    fun realNameOf(name: String): String? {
        if (!name.startsWith(prefix = ".") || !name.endsWith(suffix = ICLOUD_SUFFIX)) {
            return null
        }
        return name.substring(startIndex = 1, endIndex = name.length - ICLOUD_SUFFIX.length).takeIf { it.isNotEmpty() }
    }
}
