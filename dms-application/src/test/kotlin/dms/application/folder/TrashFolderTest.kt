package dms.application.folder

import dms.application.TestLibrary
import dms.application.common.OperationOutcome
import dms.domain.library.model.RelativePathModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue

class TrashFolderTest {

    private val energy = RelativePathModel.of(raw = "home/energy")

    @Test
    fun `a confirmed folder goes to the trash in every folder and its documents leave the catalog`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.add(path = "home/energy/power.pdf", content = "power")
            library.add(path = "home/energy/old/gas.pdf", content = "gas")
            library.add(path = "home/water.pdf", content = "water")

            val outcome = library.trashFolder(path = energy, confirmed = true)

            assertIs<OperationOutcome.Done>(value = outcome)
            library.allFolders.forEach { folder ->
                assertEquals(expected = setOf("home/water.pdf"), actual = library.folders.paths(folder = folder))
                assertTrue(actual = folder.id to energy in library.folders.trashed)
            }
            val left = library.catalog.documents.values.map { it.path.value }
            assertEquals(expected = listOf("home/water.pdf"), actual = left)
        }
    }

    @Test
    fun `nothing happens without confirmation, and the top level cannot be deleted`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.add(path = "home/energy/power.pdf", content = "power")

            val unconfirmed = library.trashFolder(path = energy, confirmed = false)
            val topLevel = library.trashFolder(path = RelativePathModel.ROOT, confirmed = true)

            assertEquals(expected = OperationOutcome.NotConfirmed, actual = unconfirmed)
            assertEquals(expected = OperationOutcome.NotFound, actual = topLevel)
            assertEquals(expected = 1, actual = library.catalog.documents.size)
            assertTrue(actual = library.folders.trashed.isEmpty())
        }
    }

    @Test
    fun `an offline sync folder gets the trashing from the journal`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.add(path = "home/energy/power.pdf", content = "power")
            library.folders.unreachable += library.syncB.id

            val outcome = assertIs<OperationOutcome.Done>(value = library.trashFolder(path = energy, confirmed = true))

            assertEquals(expected = setOf(library.syncB.id), actual = outcome.pendingFolders)
            assertTrue(actual = library.catalog.documents.isEmpty())
        }
    }
}
