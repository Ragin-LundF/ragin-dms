package dms.application.ask

import dms.application.TestLibrary
import dms.application.ask.model.AskOutcome
import dms.application.ask.model.RelatedOutcome
import dms.application.catalog.model.CatalogEdit
import dms.application.port.storage.model.FolderOperation
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.document.model.DocumentModel
import dms.domain.query.model.DocumentQueryModel
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class AskLibraryTest {

    private suspend fun libraryWithTexts(): Triple<TestLibrary, DocumentModel, DocumentModel> {
        val library = TestLibrary()
        library.open()
        val power = library.add(path = "power.pdf", content = "a")
        val car = library.add(path = "car.pdf", content = "b")
        library.reader.texts = mapOf(
            "power.pdf" to "Sample Energy electricity bill, meter reading and power used in March",
            "car.pdf" to "Car insurance policy for the family car",
        )
        return Triple(library, power, car)
    }

    @Test
    fun `indexing writes one entry per document everywhere and skips what is current`() {
        runTest {
            val (library, power, _) = libraryWithTexts()

            val first = library.indexLibrary().toList()
            val second = library.indexLibrary().toList()

            assertEquals(expected = listOf(true, true), actual = first.map { it.indexed })
            assertEquals(expected = listOf(false, false), actual = second.map { it.indexed })
            library.allFolders.forEach { folder ->
                assertNotNull(actual = library.index.read(folder = folder, documentId = power.id))
            }
        }
    }

    @Test
    fun `a changed title, another model or an offline folder are handled`() {
        runTest {
            val (library, power, _) = libraryWithTexts()
            library.indexLibrary().toList()
            library.editCatalog(
                edit = CatalogEdit.UpdateMetadata(
                    documentId = power.id,
                    metadata = DocumentMetadataModel(title = "Power"),
                ),
            )
            library.folders.unreachable += library.syncB.id

            val afterTitle = library.indexLibrary().toList()
            library.embeddings.model = "other-embed"
            val afterModel = library.indexLibrary().toList()

            // By path: car.pdf stays as it was, power.pdf has a new title.
            assertEquals(expected = listOf(false, true), actual = afterTitle.map { it.indexed })
            assertEquals(expected = listOf(true, true), actual = afterModel.map { it.indexed })
            assertTrue(
                actual = library.journal.pending().map { it.operation }
                    .contains(element = FolderOperation.CopyIndex(documentId = power.id)),
            )
        }
    }

    @Test
    fun `without the embedding server indexing stops with the reason`() {
        runTest {
            val (library, _, _) = libraryWithTexts()
            library.embeddings.available = false

            val progress = library.indexLibrary().toList()

            assertEquals(expected = 1, actual = progress.size)
            assertEquals(expected = "offline", actual = progress.single().failure)
        }
    }

    @Test
    fun `the closest passages go to the chat model and citations map back to documents`() {
        runTest {
            val (library, power, _) = libraryWithTexts()
            library.indexLibrary().toList()
            library.libraryAssistant.answer = "The meter showed it [1]; nothing else [7]."

            val outcome = assertIs<AskOutcome.Answered>(
                value = library.askLibrary(question = "How much electricity did the meter show?"),
            )

            assertEquals(expected = power.id, actual = outcome.citations.single().documentId)
            assertEquals(expected = 1, actual = outcome.citations.single().number)
            assertTrue(actual = outcome.usedVectors)
            val sources = library.libraryAssistant.sources.single()
            assertTrue(actual = sources.first().text.contains(other = "electricity bill"))
            assertTrue(
                actual = sources.first().title == power.path.fileName,
            )
        }
    }

    @Test
    fun `without an index the words of the question still find passages`() {
        runTest {
            val (library, _, car) = libraryWithTexts()
            library.embeddings.available = false

            val outcome = assertIs<AskOutcome.Answered>(
                value = library.askLibrary(question = "Which insurance policy do we have?"),
            )

            assertEquals(expected = car.id, actual = outcome.citations.single().documentId)
            assertEquals(expected = false, actual = outcome.usedVectors)
        }
    }

    @Test
    fun `nothing fitting is an honest not found without asking the AI`() {
        runTest {
            val (library, _, _) = libraryWithTexts()
            library.indexLibrary().toList()

            val outcome = library.askLibrary(
                question = "doctor",
                scope = DocumentQueryModel(text = "no document has this"),
            )

            assertEquals(expected = AskOutcome.NotFound, actual = outcome)
            assertTrue(actual = library.libraryAssistant.sources.isEmpty())
        }
    }

    @Test
    fun `related documents are the ones closest in meaning`() {
        runTest {
            val (library, power, car) = libraryWithTexts()
            val meter = library.add(path = "meter.pdf", content = "c")
            library.reader.texts = library.reader.texts + ("meter.pdf" to "Electricity meter reading, power")
            library.indexLibrary().toList()

            val related = assertIs<RelatedOutcome.Found>(value = library.relatedDocuments(documentId = power.id))
            assertEquals(expected = listOf(meter.id), actual = related.documentIds)
            assertTrue(actual = car.id !in related.documentIds)
        }
    }
}
