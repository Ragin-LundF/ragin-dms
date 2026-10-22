package dms.application.restructure

import dms.application.TestLibrary
import dms.application.common.OperationOutcome
import dms.application.port.storage.model.FolderOperation
import dms.application.restructure.model.PlanOutcome
import dms.application.restructure.model.RestructureOutcome
import dms.application.restructure.model.RestructureScope
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.restructure.FileNaming
import dms.domain.restructure.model.RestructurePlanModel
import dms.domain.restructure.template.TemplateProblem
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.test.runTest
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class RestructureTest {

    private val bills = CategoryModel(id = CategoryIdModel(value = "bills"), name = "Bills")

    private suspend fun libraryWith(count: Int): Pair<TestLibrary, List<DocumentModel>> {
        val library = TestLibrary()
        library.open()
        library.store.commit { it.addCategory(category = bills) }
        val documents = (1..count).map { index ->
            val document = library.add(path = "loose/bill-$index.pdf", content = "bill $index")
            library.store.commit { catalog ->
                catalog.assignCategory(documentId = document.id, categoryId = bills.id).andThen {
                    it.updateMetadata(
                        documentId = document.id,
                        metadata = DocumentMetadataModel(documentDate = LocalDate.of(2024, 1, index % 28 + 1)),
                    )
                }
            }
            document
        }
        return library to documents
    }

    private suspend fun plan(library: TestLibrary, template: String = "{category}/{year}"): RestructurePlanModel {
        val outcome = library.planRestructure(
            scope = RestructureScope.Folder(path = RelativePathModel.of(raw = "loose")),
            template = template,
            naming = FileNaming.KEEP,
        )
        return assertIs<PlanOutcome.Planned>(value = outcome).plan
    }

    @Test
    fun `an invalid template is refused before anything is planned`() {
        runTest {
            val (library, _) = libraryWith(count = 1)

            val outcome = library.planRestructure(
                scope = RestructureScope.Folder(path = RelativePathModel.ROOT),
                template = "{nonsense}",
                naming = FileNaming.KEEP,
            )

            assertEquals(
                expected = PlanOutcome.InvalidTemplate(problem = TemplateProblem.UNKNOWN_PLACEHOLDER),
                actual = outcome,
            )
        }
    }

    @Test
    fun `applying moves every file in every folder and records the plan`() {
        runTest {
            val (library, documents) = libraryWith(count = 3)

            val outcome = assertIs<RestructureOutcome.Applied>(
                value = library.applyRestructurePlan(plan = plan(library = library)),
            )

            assertEquals(expected = 3, actual = outcome.moved)
            library.allFolders.forEach { folder ->
                assertEquals(
                    expected = setOf("Bills/2024/bill-1.pdf", "Bills/2024/bill-2.pdf", "Bills/2024/bill-3.pdf"),
                    actual = library.folders.paths(folder = folder),
                )
                assertNotNull(actual = library.plans.recordsIn(folder = folder)[outcome.record])
            }
            assertEquals(
                expected = "Bills/2024/bill-1.pdf",
                actual = library.catalog.documents.getValue(documents.first().id).path.value,
            )
        }
    }

    @Test
    fun `the catalog is committed in batches of fifty`() {
        runTest {
            val (library, _) = libraryWith(count = 120)
            val before = library.catalog.revision

            library.applyRestructurePlan(plan = plan(library = library))

            // Three batch commits, plus none for the plan file, which is no catalog change.
            assertEquals(expected = before.value + 3, actual = library.catalog.revision.value)
        }
    }

    @Test
    fun `an offline sync folder gets its moves from the journal`() {
        runTest {
            val (library, _) = libraryWith(count = 2)
            library.folders.unreachable += library.syncB.id

            val outcome = assertIs<RestructureOutcome.Applied>(
                value = library.applyRestructurePlan(plan = plan(library = library)),
            )

            assertEquals(expected = setOf(library.syncB.id), actual = outcome.pendingFolders)
            assertEquals(
                expected = 2,
                actual = library.journal.pending().count { it.operation is FolderOperation.Move },
            )
            assertTrue(actual = library.journal.pending().any { it.operation is FolderOperation.CopyPlan })
            assertEquals(
                expected = setOf("loose/bill-1.pdf", "loose/bill-2.pdf"),
                actual = library.folders.paths(folder = library.syncB),
            )
        }
    }

    @Test
    fun `cancelling midway commits what was moved, so the catalog matches the disk`() {
        runTest {
            val (library, _) = libraryWith(count = 4)
            val plan = plan(library = library)

            assertFailsWith<CancellationException> {
                library.applyRestructurePlan(plan = plan) { done, _ ->
                    if (done == 2) {
                        throw CancellationException("stop")
                    }
                }
            }

            val moved = library.catalog.documents.values.count { it.path.value.startsWith(prefix = "Bills/") }
            assertEquals(expected = 2, actual = moved)
            library.catalog.documents.values.forEach { document ->
                assertNotNull(actual = library.folders.content(folder = library.primary, path = document.path.value))
            }
            assertEquals(expected = 1, actual = library.plans.recordsIn(folder = library.primary).size)
        }
    }

    @Test
    fun `reversing puts documents back, except one the user moved since`() {
        runTest {
            val (library, documents) = libraryWith(count = 3)
            val applied = assertIs<RestructureOutcome.Applied>(
                value = library.applyRestructurePlan(plan = plan(library = library)),
            )
            library.moveDocument(
                documentId = documents.first().id,
                newPath = RelativePathModel.of(raw = "kept/bill-1.pdf"),
            )

            val reversed = assertIs<RestructureOutcome.Applied>(
                value = library.reverseRestructure(name = assertNotNull(actual = applied.record)),
            )

            assertEquals(expected = 2, actual = reversed.moved)
            assertEquals(expected = 1, actual = reversed.skipped)
            assertEquals(
                expected = setOf("kept/bill-1.pdf", "loose/bill-2.pdf", "loose/bill-3.pdf"),
                actual = library.catalog.documents.values.map { it.path.value }.toSet(),
            )
        }
    }

    @Test
    fun `a move whose document changed in the meantime fails and the rest goes on`() {
        runTest {
            val (library, documents) = libraryWith(count = 2)
            val plan = plan(library = library)
            library.moveDocument(
                documentId = documents.first().id,
                newPath = RelativePathModel.of(raw = "elsewhere.pdf"),
            )

            val outcome = assertIs<RestructureOutcome.Applied>(value = library.applyRestructurePlan(plan = plan))

            assertEquals(expected = 1, actual = outcome.moved)
            assertEquals(expected = listOf(documents.first().id), actual = outcome.failed.map { it.documentId })
        }
    }

    @Test
    fun `empty folders are only removed with confirmation`() {
        runTest {
            val (library, _) = libraryWith(count = 1)
            library.createFolder(path = RelativePathModel.of(raw = "old"))

            assertEquals(
                expected = OperationOutcome.NotConfirmed,
                actual = library.removeEmptyFolders(
                    folders = listOf(RelativePathModel.of(raw = "old")),
                    confirmed = false,
                ),
            )
            assertIs<OperationOutcome.Done>(
                value = library.removeEmptyFolders(
                    folders = listOf(RelativePathModel.of(raw = "old"), RelativePathModel.of(raw = "loose")),
                    confirmed = true,
                ),
            )
            library.allFolders.forEach {
                assertTrue(actual = !library.folders.hasDirectory(folder = it, path = "old"))
                assertTrue(actual = library.folders.paths(folder = it).contains(element = "loose/bill-1.pdf"))
            }
        }
    }
}
