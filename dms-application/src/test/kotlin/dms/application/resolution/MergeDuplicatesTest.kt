package dms.application.resolution

import dms.application.TestLibrary
import dms.application.common.OperationOutcome
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.TagNameModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue

class MergeDuplicatesTest {

    @Test
    fun `keeping one copy trashes the other everywhere and keeps what was assigned to it`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val kept = library.add(path = "letters/a.pdf", content = "alpha")
            val copy = library.add(path = "letters/b.pdf", content = "alpha")
            library.store.commit { catalog ->
                catalog.setTags(documentId = copy.id, newTags = setOf(TagNameModel.of(raw = "energy")))
                    .andThen {
                        it.updateMetadata(documentId = copy.id, metadata = DocumentMetadataModel(title = "Bill"))
                    }
            }

            val refused = library.mergeDuplicates(keep = kept.id, confirmed = false)
            val outcome = library.mergeDuplicates(keep = kept.id, confirmed = true)

            assertIs<OperationOutcome.NotConfirmed>(value = refused)
            assertIs<OperationOutcome.Done>(value = outcome)
            val document = library.catalog.documents.values.single()
            assertEquals(expected = kept.id, actual = document.id)
            assertEquals(expected = setOf("energy"), actual = document.tags.map { it.value }.toSet())
            assertEquals(expected = "Bill", actual = document.metadata.title)
            val trashed = library.allFolders.map { it.id to RelativePathModel.of(raw = "letters/b.pdf") }.toSet()
            assertEquals(expected = trashed, actual = library.folders.trashed.toSet())
        }
    }

    @Test
    fun `nothing changes when the copy in the primary differs byte for byte`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val kept = library.add(path = "letters/a.pdf", content = "alpha")
            library.add(path = "letters/b.pdf", content = "alpha")
            library.folders.put(folder = library.primary, path = "letters/b.pdf", content = "alphb")

            val outcome = library.mergeDuplicates(keep = kept.id, confirmed = true)

            assertIs<OperationOutcome.ScanOutdated>(value = outcome)
            assertTrue(actual = library.folders.trashed.isEmpty())
            assertEquals(expected = 2, actual = library.catalog.documents.size)
        }
    }

    @Test
    fun `a sync folder whose copy differs keeps its file`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val kept = library.add(path = "letters/a.pdf", content = "alpha")
            library.add(path = "letters/b.pdf", content = "alpha")
            library.folders.put(folder = library.syncA, path = "letters/b.pdf", content = "alphb")

            val outcome = library.mergeDuplicates(keep = kept.id, confirmed = true)

            assertIs<OperationOutcome.Done>(value = outcome)
            val different = library.folders.content(folder = library.syncA, path = "letters/b.pdf")
            assertEquals(expected = "alphb", actual = different)
            val copyPath = RelativePathModel.of(raw = "letters/b.pdf")
            val trashed = listOf(library.primary, library.syncB).map { it.id to copyPath }
            assertEquals(expected = trashed.toSet(), actual = library.folders.trashed.toSet())
        }
    }

    @Test
    fun `the copy with the most assigned values is kept by default`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val plain = library.add(path = "letters/a.pdf", content = "alpha")
            val tagged = library.add(path = "letters/b.pdf", content = "alpha")
            library.store.commit {
                it.setTags(documentId = tagged.id, newTags = setOf(TagNameModel.of(raw = "energy")))
            }
            val documents = library.catalog.documents.values.filter { it.id in setOf(plain.id, tagged.id) }

            assertEquals(expected = tagged.id, actual = DuplicatePreference.preferred(copies = documents).id)
        }
    }
}
