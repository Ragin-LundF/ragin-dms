package dms.storage.catalog

import tools.jackson.databind.JsonNode

/**
 * Brings catalog files written by older versions up to [CURRENT_VERSION]. Every change of the stored
 * structure that is not just a new optional field bumps the version and adds a step here, with a test
 * that reads a file in the old format.
 */
object CatalogMigrations {

    const val CURRENT_VERSION: Int = 1

    /** The migrated content, or `null` if the file is newer than this version understands or has no version. */
    fun migrate(tree: JsonNode): JsonNode? {
        val version = tree.get("formatVersion")?.asInt() ?: return null
        if (version !in 1..CURRENT_VERSION) {
            return null
        }
        return tree
    }
}
