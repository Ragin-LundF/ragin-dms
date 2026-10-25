package dms.storage.catalog

import dms.application.port.storage.model.CatalogLoad
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.catalog.CatalogUpdate
import dms.domain.catalog.model.CatalogModel
import dms.domain.catalog.model.LibraryIdModel
import dms.domain.document.ClassificationSource
import dms.domain.document.model.ClassificationRecordModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.document.model.DocumentModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import dms.domain.taxonomy.model.TagNameModel
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
import kotlin.test.assertNotNull
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

            save(catalog = catalog)

            assertEquals(expected = CatalogLoad.Found(catalog = catalog), actual = repository.load(folder = folder))
        }
    }

    @Test
    fun `saving the same catalog twice writes the same sorted bytes`() {
        runTest {
            val catalog = sampleCatalog()

            save(catalog = catalog)
            val first = Files.readAllBytes(file)
            save(catalog = catalog)

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
                documentId = DocumentIdModel(value = "doc-1"),
                metadata = DocumentMetadataModel(title = "Changed"),
            )
            save(catalog = assertIs<CatalogUpdate.Applied>(value = update).catalog)

            val text = Files.readString(file)
            listOf(
                "futureField: kept at the top",
                "futureNote: kept in the document",
                "futureLayout: kept in the layout",
                "futureInbox: kept in the inbox",
                "futureDefinition: kept in the field definition",
                "futureSearch: kept in the saved search",
                "futureQuery: kept in the query",
                "futureCorrespondent: kept in the correspondent",
                "futureRule: kept in the rule",
                "futureAction: kept in the actions",
                "futureRetention: kept in the retention",
                "futureLink: kept in the link",
            ).forEach { kept -> assertTrue(actual = text.contains(other = kept), message = kept) }
            assertTrue(actual = text.contains(other = "title: Changed"))
        }
    }

    @Test
    fun `a catalog based on an older revision does not overwrite a newer file`() {
        runTest {
            val base = sampleCatalog()
            val newer = assertIs<CatalogUpdate.Applied>(value = base.addTag(name = "other device")).catalog
            save(catalog = newer)
            val written = Files.readString(file)
            val stale = assertIs<CatalogUpdate.Applied>(value = base.addTag(name = "this device")).catalog

            val result = repository.save(folder = folder, catalog = stale, basedOn = base.revision)

            assertEquals(expected = StorageFailure.CATALOG_CHANGED, actual = StorageResults.failureOf(result = result))
            assertEquals(expected = written, actual = Files.readString(file))
        }
    }

    @Test
    fun `a catalog based on the revision on disk is written`() {
        runTest {
            val base = sampleCatalog()
            save(catalog = base)
            val next = assertIs<CatalogUpdate.Applied>(value = base.addTag(name = "this device")).catalog

            val result = repository.save(folder = folder, catalog = next, basedOn = base.revision)

            assertIs<StorageResult.Success<*>>(value = result)
            assertEquals(expected = CatalogLoad.Found(catalog = next), actual = repository.load(folder = folder))
        }
    }

    @Test
    fun `a catalog of a newer format is neither read nor overwritten`() {
        runTest {
            val newer = catalogWithFutureFields().replace(oldValue = "formatVersion: 1", newValue = "formatVersion: 99")
            Files.writeString(file, newer)

            val load = repository.load(folder = folder)
            val save = save(catalog = sampleCatalog())

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
            val save = save(catalog = sampleCatalog())

            assertEquals(expected = StorageFailure.UNSUPPORTED_FORMAT, actual = failureOf(load = load))
            assertIs<StorageResult.Failure>(value = save)
            assertEquals(expected = broken, actual = Files.readString(file))
        }
    }

    @Test
    fun `readable conflict copies are loaded and discarded, unreadable ones stay`() {
        runTest {
            val older = sampleCatalog()
            save(catalog = older)
            Files.copy(file, root.resolve("dms-catalog 2.yaml"))
            val newer = assertIs<CatalogUpdate.Applied>(value = older.addTag(name = "extra")).catalog
            save(catalog = newer)
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

    /** Saves [catalog] as a change based on its own revision, the case of every save in this test. */
    private suspend fun save(catalog: CatalogModel): StorageResult<Unit> {
        return repository.save(folder = folder, catalog = catalog, basedOn = catalog.revision)
    }

    private fun failureOf(load: CatalogLoad): StorageFailure? {
        return (load as? CatalogLoad.Failed)?.failure?.reason
    }

    private fun CatalogModel.addTag(name: String): CatalogUpdate {
        return setTags(documentId = DocumentIdModel(value = "doc-a"), newTags = setOf(TagNameModel.of(raw = name)))
    }

    private fun sampleCatalog(): CatalogModel {
        val category = CategoryModel(id = CategoryIdModel(value = "cat-1"), name = "Invoices")
        val later = DocumentModel(
            id = DocumentIdModel(value = "doc-b"),
            path = TestFolders.path(raw = "letters/b.pdf"),
            contentHash = TestFolders.hashOf(content = "beta"),
            size = 4,
            addedAt = Instant.parse("2024-02-01T09:30:00Z"),
            tags = setOf(TagNameModel.of(raw = "Tax"), TagNameModel.of(raw = "2024")),
        )
        val earlier = DocumentModel(
            id = DocumentIdModel(value = "doc-a"),
            path = TestFolders.path(raw = "a.pdf"),
            contentHash = TestFolders.hashOf(content = "alpha"),
            size = 5,
            addedAt = Instant.parse("2024-01-01T10:00:00Z"),
            metadata = DocumentMetadataModel(
                title = "Sample invoice",
                documentDate = LocalDate.parse("2023-12-24"),
                correspondent = "Example Supplier",
            ),
            categoryId = category.id,
            classification = ClassificationRecordModel(
                source = ClassificationSource.AI,
                model = "test-model",
                confirmedAt = Instant.parse("2024-01-02T08:00:00Z"),
            ),
        )
        val update = CatalogModel.empty(libraryId = LibraryIdModel(value = "library-1"))
            .addCategory(category = category)
            .andThen { it.addDocument(document = later) }
            .andThen { it.addDocument(document = earlier) }
        return assertIs<CatalogUpdate.Applied>(value = update).catalog
    }

    /** A catalog with fields only a newer version knows, in every part that keeps them. */
    private fun catalogWithFutureFields(): String {
        val resource = assertNotNull(actual = javaClass.getResourceAsStream("/catalog/future-fields.yaml"))
        return resource.use { String(it.readAllBytes(), Charsets.UTF_8) }
    }
}
