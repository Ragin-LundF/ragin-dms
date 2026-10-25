package dms.storage.catalog

import dms.application.port.storage.model.CatalogLoad
import dms.domain.catalog.CatalogUpdate
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentIdModel
import dms.storage.support.RecordingTrash
import dms.storage.support.TestFolders
import kotlinx.coroutines.test.runTest
import java.nio.file.Files
import java.nio.file.Path
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

/** Catalog files as the released versions write them, from `src/test/resources/catalog/`. */
class CatalogFixturesTest {

    private val temp: Path = Files.createTempDirectory("dms-storage-")
    private val root: Path = Files.createDirectories(temp.resolve("library"))
    private val folder = TestFolders.folder(location = root)
    private val file: Path = root.resolve(CatalogFiles.FILE_NAME)
    private val repository = FileCatalogRepository(trash = RecordingTrash(directory = temp.resolve("trash")))

    @AfterTest
    fun deleteTemp() {
        temp.toFile().deleteRecursively()
    }

    @Test
    fun `a file written by 1_0 loads`() {
        runTest {
            val catalog = load(fixture = V1_0)

            assertEquals(expected = 2, actual = catalog.documents.size)
            assertEquals(expected = setOf("Invoices"), actual = catalog.categories.values.map { it.name }.toSet())
            assertEquals(expected = 3, actual = catalog.revision.value)
        }
    }

    @Test
    fun `a file written by 1_1 loads with every 1_1 section`() {
        runTest {
            val catalog = load(fixture = V1_1)

            val settings = catalog.settings
            assertEquals(expected = 6, actual = settings.fieldDefinitions.size)
            assertEquals(expected = 1, actual = settings.savedSearches.size)
            assertEquals(expected = 1, actual = settings.rules.size)
            assertEquals(expected = 1, actual = settings.correspondents.size)
            assertTrue(actual = settings.layout.inbox.enabled)
            assertEquals(expected = 1, actual = catalog.links.size)
            assertNotNull(actual = catalog.documents.getValue(DocumentIdModel(value = "doc-b")).retention)
        }
    }

    @Test
    fun `each fixture round-trips unchanged, to the byte`() {
        runTest {
            listOf(V1_0, V1_1).forEach { fixture ->
                val original = fixtureText(name = fixture)
                val catalog = load(fixture = fixture)

                repository.save(folder = folder, catalog = catalog, basedOn = catalog.revision)

                assertEquals(expected = original, actual = Files.readString(file), message = fixture)
                val reloaded = assertIs<CatalogLoad.Found>(value = repository.load(folder = folder)).catalog
                assertEquals(expected = catalog, actual = reloaded, message = fixture)
            }
        }
    }

    @Test
    fun `a 1_0 file edited by 1_1 keeps every unchanged line`() {
        runTest {
            val original = fixtureText(name = V1_0)
            val catalog = load(fixture = V1_0)
            val update = catalog.setNote(documentId = DocumentIdModel(value = "doc-b"), note = "Added by 1.1")
            val edited = assertIs<CatalogUpdate.Applied>(value = update).catalog

            repository.save(folder = folder, catalog = edited, basedOn = catalog.revision)

            val written = Files.readString(file).lines()
            val kept = original.lines().filterNot { it.startsWith(prefix = "revision:") }
            assertEquals(expected = kept, actual = written.filter { it in kept })
            assertTrue(actual = "revision: 4" in written)
            assertTrue(actual = "  note: Added by 1.1" in written)
        }
    }

    private suspend fun load(fixture: String): CatalogModel {
        Files.writeString(file, fixtureText(name = fixture))
        return assertIs<CatalogLoad.Found>(value = repository.load(folder = folder)).catalog
    }

    private fun fixtureText(name: String): String {
        val resource = assertNotNull(actual = javaClass.getResourceAsStream("/catalog/$name"), message = name)
        return resource.use { String(it.readAllBytes(), Charsets.UTF_8) }
    }

    private companion object {
        const val V1_0 = "format-1-v1.0.yaml"
        const val V1_1 = "format-1-v1.1.yaml"
    }
}
