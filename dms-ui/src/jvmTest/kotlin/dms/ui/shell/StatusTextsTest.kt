package dms.ui.shell

import dms.application.library.model.ScanResultModel
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.library.FolderRole
import dms.domain.library.model.FolderLocationModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.LibraryFoldersModel
import dms.ui.session.model.LibraryStatus
import dms.ui.session.model.SessionStateModel
import java.time.Instant
import java.time.ZoneOffset
import kotlin.test.Test
import kotlin.test.assertEquals

class StatusTextsTest {

    private val now = Instant.parse("2026-10-25T14:00:00Z")
    private val folders = LibraryFoldersModel(
        folders = listOf(
            folder(id = "primary", role = FolderRole.PRIMARY),
            folder(id = "sync", role = FolderRole.SYNC),
        ),
    )
    private val open = SessionStateModel(
        status = LibraryStatus.Open(foreignFolders = emptySet(), unreachableFolders = emptySet()),
        folders = folders,
    )

    @Test
    fun `an open library names its folders and the time of today's scan`() {
        val state = open.copy(lastScanAt = Instant.parse("2026-10-25T09:05:00Z"))

        assertEquals(
            expected = "Library ready · 2 folders, all online · last scan 09:05",
            actual = StatusTexts.library(state = state, now = now, zone = ZoneOffset.UTC),
        )
    }

    @Test
    fun `offline folders and an older scan are named`() {
        val unreachable = StorageResult.Failure(reason = StorageFailure.FOLDER_UNREACHABLE)
        val failed = mapOf(LibraryFolderIdModel(value = "sync") to unreachable)
        val state = open.copy(
            lastScan = ScanResultModel(changes = emptyList(), scannedFolders = emptySet(), failedFolders = failed),
            lastScanAt = Instant.parse("2026-10-22T09:05:00Z"),
        )

        assertEquals(
            expected = "Library ready · 2 folders, 1 offline · last scan 3 days ago",
            actual = StatusTexts.library(state = state, now = now, zone = ZoneOffset.UTC),
        )
    }

    @Test
    fun `without a library only the state is named`() {
        val state = SessionStateModel(status = LibraryStatus.NotConfigured)

        assertEquals(
            expected = "No library yet",
            actual = StatusTexts.library(state = state, now = now, zone = ZoneOffset.UTC),
        )
    }

    private fun folder(id: String, role: FolderRole): LibraryFolderModel {
        return LibraryFolderModel(
            id = LibraryFolderIdModel(value = id),
            displayName = id,
            location = FolderLocationModel(value = "/library/$id"),
            role = role,
        )
    }
}
