package dms.application.library

import dms.application.TestLibrary
import dms.application.common.OperationOutcome
import dms.application.port.storage.model.StorageFailure
import dms.domain.catalog.CatalogUpdate
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class CatalogStoreTest {

    private val mine = TestLibrary.document(id = "mine", path = "letters/mine.pdf", content = "mine")
    private val theirs = TestLibrary.document(id = "theirs", path = "letters/theirs.pdf", content = "theirs")
    private val theirsToo = TestLibrary.document(id = "theirs-2", path = "letters/theirs-2.pdf", content = "theirs 2")

    @Test
    fun `a newer catalog another device saved is merged and the edit is applied on top`() {
        runTest {
            val library = TestLibrary()
            val opened = library.open()
            library.folders.catalogs[library.primary.id] = added(catalog = opened, document = theirs)

            val outcome = library.store.commit { it.addDocument(document = mine) }

            val done = assertIs<OperationOutcome.Done>(value = outcome)
            assertEquals(expected = setOf(mine.id, theirs.id), actual = done.catalog.documents.keys)
            library.allFolders.forEach { folder ->
                assertEquals(expected = done.catalog, actual = library.folders.catalogs[folder.id])
            }
            assertEquals(expected = done.catalog, actual = library.catalog)
        }
    }

    @Test
    fun `a second refused save is reported and the newer catalog stays`() {
        runTest {
            val library = TestLibrary()
            val opened = library.open()
            library.folders.beforeSave = { folder ->
                if (folder == library.primary) {
                    val onDisk = assertNotNull(actual = library.folders.catalogs[folder.id])
                    library.folders.catalogs[folder.id] = onDisk.copy(revision = onDisk.revision.next().next())
                }
            }

            val outcome = library.store.commit { it.addDocument(document = mine) }

            val failed = assertIs<OperationOutcome.StorageFailed>(value = outcome)
            assertEquals(expected = StorageFailure.CATALOG_CHANGED, actual = failed.failure.reason)
            val primary = assertNotNull(actual = library.folders.catalogs[library.primary.id])
            assertTrue(actual = mine.id !in primary.documents)
            assertTrue(actual = primary.revision > opened.revision)
        }
    }

    @Test
    fun `a sync folder holding a newer catalog is not overwritten`() {
        runTest {
            val library = TestLibrary()
            val opened = library.open()
            val newer = added(catalog = added(catalog = opened, document = theirs), document = theirsToo)
            library.folders.catalogs[library.syncA.id] = newer

            val outcome = library.store.commit { it.addDocument(document = mine) }

            val done = assertIs<OperationOutcome.Done>(value = outcome)
            assertEquals(expected = setOf(library.syncA.id), actual = done.pendingFolders)
            assertEquals(expected = newer, actual = library.folders.catalogs[library.syncA.id])
            assertEquals(expected = done.catalog, actual = library.folders.catalogs[library.syncB.id])
        }
    }

    private fun added(catalog: CatalogModel, document: DocumentModel): CatalogModel {
        return assertIs<CatalogUpdate.Applied>(value = catalog.addDocument(document = document)).catalog
    }
}
