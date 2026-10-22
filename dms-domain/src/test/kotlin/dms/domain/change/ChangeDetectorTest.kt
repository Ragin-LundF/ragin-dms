package dms.domain.change

import dms.domain.TestData
import dms.domain.change.model.FileFactsModel
import dms.domain.change.model.FolderSnapshotModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class ChangeDetectorTest {

    private val main = LibraryFolderIdModel(value = "main")
    private val backup = LibraryFolderIdModel(value = "backup")
    private val report = TestData.document(id = "d1", path = "projects/report.pdf", hashSeed = 1, size = 10)
    private val catalog = TestData.catalogWith(report)

    @Test
    fun `folders matching the catalog produce no changes`() {
        val changes = ChangeDetector.detect(
            catalog = catalog,
            snapshots = listOf(snapshotOf(folderId = main, report), snapshotOf(folderId = backup, report)),
        )

        assertTrue(actual = changes.isEmpty())
    }

    @Test
    fun `an unhashed file with the catalogued size counts as unchanged`() {
        val unhashed = FileFactsModel(size = 10, contentHash = null)
        val snapshot = FolderSnapshotModel(folderId = main, files = mapOf(report.path to unhashed))

        assertTrue(actual = ChangeDetector.detect(catalog = catalog, snapshots = listOf(snapshot)).isEmpty())
    }

    @Test
    fun `different content in one folder is a modification of that folder`() {
        val snapshot = FolderSnapshotModel(
            folderId = backup,
            files = mapOf(report.path to FileFactsModel(size = 11, contentHash = TestData.hash(seed = 2))),
        )

        val change = ChangeDetector.detect(catalog = catalog, snapshots = listOf(snapshot)).single()

        assertEquals(
            expected = DetectedChange.ModifiedFile(
                documentId = report.id,
                path = report.path,
                folderId = backup,
                size = 11,
                contentHash = TestData.hash(seed = 2),
            ),
            actual = change,
        )
    }

    @Test
    fun `a file found under another path with the same content is a move`() {
        val moved = RelativePathModel.of(raw = "archive/report.pdf")
        val snapshot = FolderSnapshotModel(
            folderId = main,
            files = mapOf(moved to FileFactsModel(size = 10, contentHash = report.contentHash)),
        )

        val change = ChangeDetector.detect(catalog = catalog, snapshots = listOf(snapshot)).single()

        assertEquals(
            expected = DetectedChange.MovedFile(
                documentId = report.id,
                path = moved,
                previousPath = report.path,
                folderId = main,
            ),
            actual = change,
        )
    }

    @Test
    fun `a missing file names the folders that lack it and the ones that still have it`() {
        val changes = ChangeDetector.detect(
            catalog = catalog,
            snapshots = listOf(snapshotOf(folderId = main), snapshotOf(folderId = backup, report)),
        )

        assertEquals(
            expected = listOf(
                DetectedChange.MissingFile(
                    documentId = report.id,
                    path = report.path,
                    missingIn = setOf(main),
                    presentIn = setOf(backup),
                ),
            ),
            actual = changes,
        )
    }

    @Test
    fun `an unreachable folder is not reported as missing anything`() {
        val changes = ChangeDetector.detect(catalog = catalog, snapshots = listOf(snapshotOf(folderId = main, report)))

        assertTrue(actual = changes.isEmpty())
    }

    @Test
    fun `an uncatalogued file is new and lists every folder that has it`() {
        val extra = TestData.document(id = "x", path = "inbox/scan.pdf", hashSeed = 3, size = 5)

        val change = ChangeDetector.detect(
            catalog = catalog,
            snapshots = listOf(
                snapshotOf(folderId = main, report, extra),
                snapshotOf(folderId = backup, report, extra),
            ),
        ).single()

        assertEquals(
            expected = DetectedChange.NewFile(
                path = extra.path,
                presentIn = setOf(main, backup),
                size = 5,
                contentHash = extra.contentHash,
            ),
            actual = change,
        )
    }

    @Test
    fun `an uncatalogued file with different content per folder is a conflict`() {
        val path = RelativePathModel.of(raw = "inbox/scan.pdf")
        val inMain = FileFactsModel(size = 5, contentHash = TestData.hash(seed = 4))
        val inBackup = FileFactsModel(size = 6, contentHash = TestData.hash(seed = 5))
        val snapshots = listOf(
            FolderSnapshotModel(
                folderId = main,
                files = mapOf(report.path to facts(document = report), path to inMain),
            ),
            FolderSnapshotModel(
                folderId = backup,
                files = mapOf(report.path to facts(document = report), path to inBackup),
            ),
        )

        val change = ChangeDetector.detect(catalog = catalog, snapshots = snapshots).single()

        assertEquals(
            expected = DetectedChange.ContentConflict(
                path = path,
                contentHashes = mapOf(main to TestData.hash(seed = 4), backup to TestData.hash(seed = 5)),
            ),
            actual = change,
        )
    }

    @Test
    fun `changes are sorted by path`() {
        val first = TestData.document(id = "a", path = "a.pdf", hashSeed = 10)
        val last = TestData.document(id = "z", path = "z.pdf", hashSeed = 11)

        val changes = ChangeDetector.detect(
            catalog = catalog,
            snapshots = listOf(snapshotOf(folderId = main, last, first)),
        )

        assertEquals(
            expected = listOf("a.pdf", "projects/report.pdf", "z.pdf"),
            actual = changes.map { it.path.value },
        )
    }

    private fun snapshotOf(folderId: LibraryFolderIdModel, vararg documents: DocumentModel): FolderSnapshotModel {
        return FolderSnapshotModel(folderId = folderId, files = documents.associate { it.path to facts(document = it) })
    }

    private fun facts(document: DocumentModel): FileFactsModel {
        return FileFactsModel(size = document.size, contentHash = document.contentHash)
    }
}
