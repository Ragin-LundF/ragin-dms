package dms.application.library

import dms.application.TestLibrary
import dms.domain.task.OpenTask
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class BuildDashboardTest {

    @Test
    fun `scan results and failed operations become tasks`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.folders.unreachable += library.syncB.id
            library.add(path = "letters/a.pdf", content = "alpha")
            library.folders.put(folder = library.syncA, path = "inbox/new.pdf", content = "new")

            val scan = assertNotNull(actual = library.scanLibrary())
            val dashboard = assertNotNull(actual = library.buildDashboard(scan = scan))

            assertTrue(actual = dashboard.tasks.any { it is OpenTask.AdoptNewFile })
            assertTrue(
                actual = OpenTask.RetryFailedOperations(folderId = library.syncB.id, count = 2) in dashboard.tasks,
            )
            assertEquals(expected = false, actual = dashboard.folders.single { it.folder == library.syncB }.reachable)
            assertEquals(expected = 1, actual = dashboard.documentCount)
            assertEquals(expected = 1, actual = dashboard.uncategorizedCount)
        }
    }

    @Test
    fun `a dismissed task stays hidden when the scan finds the change again`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.folders.put(folder = library.syncA, path = "inbox/new.pdf", content = "new")
            val firstScan = assertNotNull(actual = library.scanLibrary())
            val task = assertNotNull(actual = library.buildDashboard(scan = firstScan)).tasks.single()

            library.dismissTask(key = task.key)
            val dashboard = assertNotNull(actual = library.buildDashboard(scan = library.scanLibrary()))

            assertTrue(actual = dashboard.tasks.isEmpty())
        }
    }
}
