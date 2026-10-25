package dms.storage.catalog

import dms.application.port.storage.model.CatalogLoad
import dms.application.port.storage.model.StorageResult
import dms.domain.catalog.CatalogUpdate
import dms.domain.catalog.model.CatalogModel
import dms.domain.catalog.model.LibraryIdModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.field.FieldPresets
import dms.domain.field.FieldType
import dms.domain.field.FieldValue
import dms.domain.field.model.CurrencyCodeModel
import dms.domain.field.model.FieldDefinitionModel
import dms.domain.field.model.FieldKeyModel
import dms.domain.library.model.RelativePathModel
import dms.domain.link.model.DocumentLinkModel
import dms.domain.query.FieldComparison
import dms.domain.query.model.DateRangeModel
import dms.domain.query.model.DocumentQueryModel
import dms.domain.query.model.FieldConditionModel
import dms.domain.query.model.SavedSearchIdModel
import dms.domain.query.model.SavedSearchModel
import dms.domain.query.model.TagFilterModel
import dms.domain.retention.model.RetentionModel
import dms.domain.rule.RuleCondition
import dms.domain.rule.RuleOrigin
import dms.domain.rule.model.CorrespondentModel
import dms.domain.rule.model.MatchingRuleIdModel
import dms.domain.rule.model.MatchingRuleModel
import dms.domain.rule.model.RuleActionsModel
import dms.domain.settings.model.CatalogSettingsModel
import dms.domain.settings.model.InboxSettingsModel
import dms.domain.settings.model.LibraryLayoutModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import dms.domain.taxonomy.model.TagNameModel
import dms.storage.support.RecordingTrash
import dms.storage.support.TestFolders
import kotlinx.coroutines.test.runTest
import java.math.BigDecimal
import java.nio.file.Files
import java.nio.file.Path
import java.time.Instant
import java.time.LocalDate
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertTrue

/** The fields added in 1.1: custom fields, notes, retention, links and the library-wide settings. */
class CatalogDetailsFormatTest {

    private val temp: Path = Files.createTempDirectory("dms-storage-")
    private val root: Path = Files.createDirectories(temp.resolve("library"))
    private val folder = TestFolders.folder(location = root)
    private val file: Path = root.resolve(CatalogFiles.FILE_NAME)
    private val repository = FileCatalogRepository(trash = RecordingTrash(directory = temp.resolve("trash")))
    private val euro = CurrencyCodeModel(value = "EUR")

    @AfterTest
    fun deleteTemp() {
        temp.toFile().deleteRecursively()
    }

    @Test
    fun `every new field survives a round trip`() {
        runTest {
            val catalog = detailedCatalog()

            save(catalog = catalog)

            assertEquals(expected = CatalogLoad.Found(catalog = catalog), actual = repository.load(folder = folder))
        }
    }

    @Test
    fun `a note is written as a block and values as plain text`() {
        runTest {
            save(catalog = detailedCatalog())

            val text = Files.readString(file)
            assertTrue(actual = text.contains(other = "note: |-\n    Paid on time.\n    Second line"))
            assertTrue(actual = text.contains(other = "amount: \"1234.50\""))
            assertTrue(actual = text.contains(other = "currency: EUR"))
        }
    }

    @Test
    fun `a library without the new fields writes none of them`() {
        runTest {
            val plain = CatalogModel.empty(libraryId = LibraryIdModel(value = "library-1"))
            save(catalog = plain)

            val text = Files.readString(file)
            listOf("layout", "fieldDefinitions", "savedSearches", "rules", "correspondents", "links").forEach {
                assertFalse(actual = text.contains(other = it), message = it)
            }
        }
    }

    @Test
    fun `a file of version 1_0 without the new fields still loads`() {
        runTest {
            val hash = TestFolders.hashOf(content = "alpha").value
            Files.writeString(
                file,
                """
                formatVersion: 1
                libraryId: library-1
                revision: 2
                categories:
                - id: cat-1
                  name: Invoices
                documents:
                - id: doc-1
                  path: a.pdf
                  contentHash: $hash
                  size: 5
                  addedAt: 2024-01-01T10:00:00Z
                  categoryId: cat-1
                """.trimIndent() + "\n",
            )

            val catalog = assertIs<CatalogLoad.Found>(value = repository.load(folder = folder)).catalog

            assertEquals(expected = CatalogSettingsModel.EMPTY, actual = catalog.settings)
            assertTrue(actual = catalog.links.isEmpty())
            assertEquals(expected = emptyMap(), actual = catalog.documents.values.single().fields)
        }
    }

    @Test
    fun `links to documents an older version removed are dropped instead of breaking the file`() {
        runTest {
            val hash = TestFolders.hashOf(content = "alpha").value
            Files.writeString(
                file,
                """
                formatVersion: 1
                libraryId: library-1
                revision: 2
                documents:
                - id: doc-1
                  path: a.pdf
                  contentHash: $hash
                  size: 5
                  addedAt: 2024-01-01T10:00:00Z
                links:
                - first: doc-0
                  second: doc-1
                """.trimIndent() + "\n",
            )

            val catalog = assertIs<CatalogLoad.Found>(value = repository.load(folder = folder)).catalog

            assertTrue(actual = catalog.links.isEmpty())
        }
    }

    @Test
    fun `unknown fields next to the new ones are still kept`() {
        runTest {
            val catalog = detailedCatalog()
            save(catalog = catalog)
            val withFuture = Files.readString(file)
                .replace(oldValue = "revision: ", newValue = "futureSetting: kept\nrevision: ")
            Files.writeString(file, withFuture)

            val update = catalog.setNote(documentId = DocumentIdModel(value = "doc-b"), note = "Changed")
            save(catalog = assertIs<CatalogUpdate.Applied>(value = update).catalog)

            val text = Files.readString(file)
            assertTrue(actual = text.contains(other = "futureSetting: kept"))
            assertTrue(actual = text.contains(other = "note: Changed"))
        }
    }

    private val category = CategoryModel(id = CategoryIdModel(value = "cat-1"), name = "Contracts", retentionYears = 10)
    private val project = FieldDefinitionModel(
        key = FieldKeyModel(value = "project"),
        label = "Project",
        type = FieldType.TEXT,
    )
    private val tax = TagNameModel.of(raw = "Tax")

    /** Saves [catalog] as a change based on its own revision, the case of every save in this test. */
    private suspend fun save(catalog: CatalogModel): StorageResult<Unit> {
        return repository.save(folder = folder, catalog = catalog, basedOn = catalog.revision)
    }

    private fun detailedCatalog(): CatalogModel {
        val first = document(id = "doc-a", path = "a.pdf", content = "alpha")
        val second = document(id = "doc-b", path = "letters/b.pdf", content = "beta")
        val update = CatalogModel.empty(libraryId = LibraryIdModel(value = "library-1"))
            .addCategory(category = category)
            .andThen { it.addDocument(document = first) }
            .andThen { it.addDocument(document = second) }
            .andThen { it.changeSettings(settings = detailedSettings()) }
            .andThen {
                it.setFields(
                    documentId = first.id,
                    fields = mapOf(
                        FieldPresets.AMOUNT.key to FieldValue.Money(amount = BigDecimal("1234.50"), currency = euro),
                        FieldPresets.CANCEL_BY.key to FieldValue.Date(date = LocalDate.of(2026, 9, 30)),
                        project.key to FieldValue.Text(text = "Home"),
                    ),
                )
            }
            .andThen { it.setNote(documentId = first.id, note = "Paid on time.\nSecond line") }
            .andThen {
                it.setRetention(
                    documentId = second.id,
                    retention = RetentionModel(
                        keepUntil = LocalDate.of(2030, 12, 31),
                        settledAt = Instant.parse("2026-02-01T08:00:00Z"),
                    ),
                )
            }
            .andThen { it.link(link = DocumentLinkModel.of(first = second.id, second = first.id, label = "reminder")) }
        return assertIs<CatalogUpdate.Applied>(value = update).catalog
    }

    private fun detailedSettings(): CatalogSettingsModel {
        return CatalogSettingsModel(
            fieldDefinitions = FieldPresets.ALL + project,
            savedSearches = listOf(savedSearch()),
            rules = listOf(rule()),
            correspondents = listOf(CorrespondentModel(name = "Example Energy", aliases = listOf("EE Ltd"))),
            layout = LibraryLayoutModel(
                inbox = InboxSettingsModel(
                    enabled = true,
                    path = RelativePathModel.of(raw = "Inbox/New"),
                    autoAnalyze = true,
                ),
                pathTemplate = "{category}/{year}",
                defaultCurrency = euro,
            ),
        )
    }

    private fun savedSearch(): SavedSearchModel {
        return SavedSearchModel(
            id = SavedSearchIdModel(value = "search-1"),
            name = "Tax 2026",
            query = DocumentQueryModel(
                text = "invoice",
                category = category.id,
                tags = TagFilterModel(all = setOf(tax), none = setOf(TagNameModel.of(raw = "Draft"))),
                dates = DateRangeModel.year(year = 2026),
                folder = RelativePathModel.of(raw = "letters"),
                fields = listOf(
                    FieldConditionModel(
                        key = FieldPresets.AMOUNT.key,
                        comparison = FieldComparison.GREATER_THAN,
                        value = FieldValue.Number(number = BigDecimal("100")),
                    ),
                    FieldConditionModel(key = FieldPresets.DUE_DATE.key, comparison = FieldComparison.PRESENT),
                ),
                addedSince = Instant.parse("2026-01-01T00:00:00Z"),
                contentText = "total",
            ),
        )
    }

    private fun rule(): MatchingRuleModel {
        return MatchingRuleModel(
            id = MatchingRuleIdModel(value = "rule-1"),
            name = "Example Energy",
            conditions = listOf(
                RuleCondition.CorrespondentIs(name = "Example Energy"),
                RuleCondition.TextContains(words = listOf("meter", "kWh"), requireAll = true),
                RuleCondition.FileNameMatches(glob = "bill-*.pdf"),
            ),
            actions = RuleActionsModel(
                categoryId = category.id,
                tags = setOf(tax),
                fields = mapOf(project.key to FieldValue.Text(text = "Home")),
                targetFolder = RelativePathModel.of(raw = "letters"),
            ),
            origin = RuleOrigin.LEARNED,
            enabled = false,
        )
    }

    private fun document(id: String, path: String, content: String): DocumentModel {
        return DocumentModel(
            id = DocumentIdModel(value = id),
            path = TestFolders.path(raw = path),
            contentHash = TestFolders.hashOf(content = content),
            size = content.length.toLong(),
            addedAt = Instant.parse("2024-01-01T10:00:00Z"),
        )
    }
}
