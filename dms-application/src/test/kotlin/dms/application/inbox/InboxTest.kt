package dms.application.inbox

import dms.application.TestLibrary
import dms.application.common.OperationOutcome
import dms.application.inbox.model.CollectInboxOutcome
import dms.application.port.storage.model.FolderOperation
import dms.domain.catalog.CatalogRejection
import dms.domain.library.model.RelativePathModel
import dms.domain.settings.model.InboxSettingsModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class InboxTest {

    private val inbox = InboxSettingsModel(enabled = true, path = RelativePathModel.of(raw = "Inbox"))

    @Test
    fun `switching the inbox on creates its folder everywhere and stores the settings in the catalog`() {
        runTest {
            val library = TestLibrary()
            library.open()

            val outcome = library.configureInbox(inbox = inbox)

            assertIs<OperationOutcome.Done>(value = outcome)
            assertEquals(expected = inbox, actual = library.catalog.settings.layout.inbox)
            library.allFolders.forEach {
                assertTrue(actual = library.folders.hasDirectory(folder = it, path = "Inbox"))
            }
            assertEquals(
                expected = inbox,
                actual = library.folders.catalogs.getValue(library.syncB.id).settings.layout.inbox,
            )
        }
    }

    @Test
    fun `an offline sync folder gets the inbox folder from the journal`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.folders.unreachable += library.syncB.id

            val outcome = assertIs<OperationOutcome.Done>(value = library.configureInbox(inbox = inbox))

            assertEquals(expected = setOf(library.syncB.id), actual = outcome.pendingFolders)
            assertTrue(
                actual = library.journal.pending().any {
                    it.operation == FolderOperation.CreateFolder(path = inbox.path)
                },
            )
        }
    }

    @Test
    fun `a file dropped into a sync folder's inbox reaches the primary and every other folder`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.configureInbox(inbox = inbox)
            library.folders.put(folder = library.syncA, path = "Inbox/scan-7.pdf", content = "dropped")
            val scan = assertNotNull(actual = library.scanned())

            val collected = assertIs<CollectInboxOutcome.Collected>(value = library.collectInbox(scan = scan)).changes

            assertEquals(expected = listOf("Inbox/scan-7.pdf"), actual = collected.map { it.path.value })
            library.allFolders.forEach {
                assertEquals(
                    expected = "dropped",
                    actual = library.folders.content(folder = it, path = "Inbox/scan-7.pdf"),
                )
            }
            assertNotNull(actual = library.catalog.findByPath(path = RelativePathModel.of(raw = "Inbox/scan-7.pdf")))
        }
    }

    @Test
    fun `new files outside the inbox are left to the dashboard`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.configureInbox(inbox = inbox)
            library.folders.put(folder = library.primary, path = "elsewhere.pdf", content = "x")
            val scan = assertNotNull(actual = library.scanned())

            assertEquals(
                expected = CollectInboxOutcome.Collected(changes = emptyList()),
                actual = library.collectInbox(scan = scan),
            )
        }
    }

    @Test
    fun `filing by hand moves the document into a new folder everywhere`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.configureInbox(inbox = inbox)
            val document = library.add(path = "Inbox/scan-1.pdf", content = "a")

            val outcome = library.fileInboxDocument(
                documentId = document.id,
                targetFolder = RelativePathModel.of(raw = "Home/Power"),
                newName = "Power bill.pdf",
            )

            assertIs<OperationOutcome.Done>(value = outcome)
            assertEquals(
                expected = "Home/Power/Power bill.pdf",
                actual = library.catalog.documents.getValue(document.id).path.value,
            )
            library.allFolders.forEach {
                assertEquals(expected = setOf("Home/Power/Power bill.pdf"), actual = library.folders.paths(folder = it))
            }
            assertTrue(
                actual = !library.catalog.settings.layout.inbox.contains(
                    candidate = library.catalog.documents.getValue(document.id).path,
                ),
            )
        }
    }

    @Test
    fun `a name taken in the target folder gets a numbered suffix`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.configureInbox(inbox = inbox)
            library.add(path = "Home/scan.pdf", content = "old")
            val document = library.add(path = "Inbox/SCAN.pdf", content = "new")

            library.fileInboxDocument(
                documentId = document.id,
                targetFolder = RelativePathModel.of(raw = "Home"),
                newName = null,
            )

            assertEquals(
                expected = "Home/SCAN (2).pdf",
                actual = library.catalog.documents.getValue(document.id).path.value,
            )
        }
    }

    @Test
    fun `filing into the inbox itself is refused`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.configureInbox(inbox = inbox)
            val document = library.add(path = "Inbox/a.pdf", content = "a")

            assertEquals(
                expected = OperationOutcome.Rejected(reason = CatalogRejection.INVALID_FOLDER_MOVE),
                actual = library.fileInboxDocument(
                    documentId = document.id,
                    targetFolder = RelativePathModel.of(raw = "Inbox/x"),
                    newName = null,
                ),
            )
        }
    }
}
