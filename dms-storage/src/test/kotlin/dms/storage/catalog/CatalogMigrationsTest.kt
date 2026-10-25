package dms.storage.catalog

import dms.storage.yaml.YamlMappers
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class CatalogMigrationsTest {

    @Test
    fun `a file of the current version is returned as it is`() {
        val tree = YamlMappers.yaml.readTree("formatVersion: ${CatalogMigrations.CURRENT_VERSION}\nlibraryId: l\n")

        assertEquals(expected = tree, actual = CatalogMigrations.migrate(tree = tree))
    }

    @Test
    fun `a file of a newer version is not read`() {
        val tree = YamlMappers.yaml.readTree("formatVersion: ${CatalogMigrations.CURRENT_VERSION + 1}\n")

        assertNull(actual = CatalogMigrations.migrate(tree = tree))
    }

    @Test
    fun `a file without a version or with version zero is not read`() {
        assertNull(actual = CatalogMigrations.migrate(tree = YamlMappers.yaml.readTree("libraryId: l\n")))
        assertNull(actual = CatalogMigrations.migrate(tree = YamlMappers.yaml.readTree("formatVersion: 0\n")))
    }
}
