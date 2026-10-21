package dms.storage.catalog

import dms.application.port.storage.model.CatalogLoad
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.catalog.Catalog
import dms.domain.catalog.CatalogUpdate
import dms.domain.catalog.LibraryId
import dms.domain.document.ClassificationRecord
import dms.domain.document.ClassificationSource
import dms.domain.document.Document
import dms.domain.document.DocumentId
import dms.domain.document.DocumentMetadata
import dms.domain.taxonomy.Category
import dms.domain.taxonomy.CategoryId
import dms.domain.taxonomy.TagName
import dms.storage.support.RecordingTrash
import dms.storage.support.StorageResults
import dms.storage.support.TestFolders
import kotlinx.coroutines.test.runTest
import java.nio.file.Files
import java.nio.file.Path
import java.time.Instant
import java.time.LocalDate
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue

class FileCatalogRepositoryTest {

    private val temp: Path = Files.createTempDirectory("dms-storage-")
    private val root: Path = Files.createDirectories(temp.resolve("library"))
    private val folder = TestFolders.folder(location = root)
    private val file: Path = root.resolve(CatalogFiles.FILE_NAME)
    private val trash = RecordingTrash(directory = temp.resolve("trash"))
    private val repository = FileCatalogRepository(trash = trash)

    @AfterTest
    fun deleteTemp() {
        temp.toFile().deleteRecursively()
    }

    @Test
    fun `a saved catalog loads unchanged`() {
        runTest {
            val catalog = sampleCatalog()

            repository.save(folder = folder, catalog = catalog)

            assertEquals(expected = CatalogLoad.Found(catalog = catalog), actual = repository.load(folder = folder))
        }
    }

    @Test
    fun `saving the same catalog twice writes the same sorted bytes`() {
        runTest {
            val catalog = sampleCatalog()

            repository.save(folder = folder, catalog = catalog)
            val first = Files.readAllBytes(file)
            repository.save(folder = folder, catalog = catalog)

            assertContentEquals(expected = first, actual = Files.readAllBytes(file))
            val text = String(first, Charsets.UTF_8)
            assertTrue(actual = text.indexOf(string = "path: a.pdf") < text.indexOf(string = "path: letters/b.pdf"))
        }
    }

    @Test
    fun `fields of a newer version are kept when saving`() {
        runTest {
            Files.writeString(file, catalogWithFutureFields())
            val loaded = assertIs<CatalogLoad.Found>(value = repository.load(folder = folder)).catalog

            val update = loaded.updateMetadata(
                documentId = DocumentId(value = "doc-1"),
                metadata = DocumentMetadata(title = "Changed"),
            )
            repository.save(folder = folder, catalog = assertIs<CatalogUpdate.Applied>(value = update).catalog)

            val text = Files.readString(file)
            assertTrue(actual = text.contains(other = "futureField: kept at the top"))
            assertTrue(actual = text.contains(other = "futureNote: kept in the document"))
            assertTrue(actual = text.contains(other = "title: Changed"))
        }
    }

    @Test
    fun `a catalog of a newer format is neither read nor overwritten`() {
        runTest {
            val newer = catalogWithFutureFields().replace(oldValue = "formatVersion: 1", newValue = "formatVersion: 99")
            Files.writeString(file, newer)

            val load = repository.load(folder = folder)
            val save = repository.save(folder = folder, catalog = sampleCatalog())

            assertEquals(expected = StorageFailure.UNSUPPORTED_FORMAT, actual = failureOf(load = load))
            assertEquals(expected = StorageFailure.UNSUPPORTED_FORMAT, actual = StorageResults.failureOf(result = save))
            assertEquals(expected = newer, actual = Files.readString(file))
        }
    }

    @Test
    fun `a broken catalog file is not overwritten`() {
        runTest {
            val broken = "formatVersion: [1\n"
            Files.writeString(file, broken)

            val load = repository.load(folder = folder)
            val save = repository.save(folder = folder, catalog = sampleCatalog())

            assertEquals(expected = StorageFailure.UNSUPPORTED_FORMAT, actual = failureOf(load = load))
            assertIs<StorageResult.Failure>(value = save)
            assertEquals(expected = broken, actual = Files.readString(file))
        }
    }

    @Test
    fun `readable conflict copies are loaded and discarded, unreadable ones stay`() {
        runTest {
            val older = sampleCatalog()
            repository.save(folder = folder, catalog = older)
            Files.copy(file, root.resolve("dms-catalog 2.yaml"))
            val newer = assertIs<CatalogUpdate.Applied>(value = older.addTag(name = "extra")).catalog
            repository.save(folder = folder, catalog = newer)
            TestFolders.write(file = root.resolve("dms-catalog (conflicted copy).yaml"), content = "revision: [")

            val load = repository.load(folder = folder)
            val discard = repository.discardConflictCopies(folder = folder)

            assertEquals(expected = CatalogLoad.Found(catalog = newer, conflictCopies = listOf(older)), actual = load)
            assertIs<StorageResult.Success<*>>(value = discard)
            assertEquals(expected = listOf("dms-catalog 2.yaml"), actual = trash.trashed)
            assertTrue(actual = Files.exists(root.resolve("dms-catalog (conflicted copy).yaml")))
        }
    }

    @Test
    fun `a folder without catalog file reports not found and an offline folder fails`() {
        runTest {
            val offline = TestFolders.folder(location = temp.resolve("offline"))

            assertEquals(expected = CatalogLoad.NotFound, actual = repository.load(folder = folder))
            assertEquals(
                expected = StorageFailure.FOLDER_UNREACHABLE,
                actual = failureOf(load = repository.load(folder = offline)),
            )
        }
    }

    private fun failureOf(load: CatalogLoad): StorageFailure? {
        return (load as? CatalogLoad.Failed)?.failure?.reason
    }

    private fun Catalog.addTag(name: String): CatalogUpdate {
        return setTags(documentId = DocumentId(value = "doc-a"), newTags = setOf(TagName.of(raw = name)))
    }

    private fun sampleCatalog(): Catalog {
        val category = Category(id = CategoryId(value = "cat-1"), name = "Invoices")
        val later = Document(
            id = DocumentId(value = "doc-b"),
            path = TestFolders.path(raw = "letters/b.pdf"),
            contentHash = TestFolders.hashOf(content = "beta"),
            size = 4,
            addedAt = Instant.parse("2024-02-01T09:30:00Z"),
            tags = setOf(TagName.of(raw = "Tax"), TagName.of(raw = "2024")),
        )
        val earlier = Document(
            id = DocumentId(value = "doc-a"),
            path = TestFolders.path(raw = "a.pdf"),
            contentHash = TestFolders.hashOf(content = "alpha"),
            size = 5,
            addedAt = Instant.parse("2024-01-01T10:00:00Z"),
            metadata = DocumentMetadata(
                title = "Sample invoice",
                documentDate = LocalDate.parse("2023-12-24"),
                correspondent = "Example Supplier",
            ),
            categoryId = category.id,
            classification = ClassificationRecord(
                source = ClassificationSource.AI,
                model = "test-model",
                confirmedAt = Instant.parse("2024-01-02T08:00:00Z"),
            ),
        )
        val update = Catalog.empty(libraryId = LibraryId(value = "library-1"))
            .addCategory(category = category)
            .andThen { it.addDocument(document = later) }
            .andThen { it.addDocument(document = earlier) }
        return assertIs<CatalogUpdate.Applied>(value = update).catalog
    }

    private fun catalogWithFutureFields(): String {
        val hash = TestFolders.hashOf(content = "alpha").value
        return """
            formatVersion: 1
            libraryId: library-1
            revision: 3
            futureField: kept at the top
            documents:
            - id: doc-1
              path: a.pdf
              contentHash: $hash
              size: 5
              addedAt: 2024-01-01T10:00:00Z
              futureNote: kept in the document
        """.trimIndent() + "\n"
    }
}
