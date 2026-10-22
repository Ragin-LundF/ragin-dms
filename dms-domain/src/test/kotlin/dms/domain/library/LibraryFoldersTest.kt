package dms.domain.library

import dms.domain.library.model.FolderLocationModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.LibraryFoldersModel
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class LibraryFoldersTest {

    private val main = folder(id = "main", role = FolderRole.PRIMARY)
    private val backup = folder(id = "backup", role = FolderRole.SYNC)
    private val share = folder(id = "share", role = FolderRole.SYNC)
    private val folders = LibraryFoldersModel(folders = listOf(main, backup))

    @Test
    fun `primary and sync folders are separated`() {
        assertEquals(expected = main, actual = folders.primary)
        assertEquals(expected = listOf(backup), actual = folders.syncFolders)
    }

    @Test
    fun `exactly one primary is required`() {
        assertFailsWith<IllegalArgumentException> { LibraryFoldersModel(folders = listOf(backup)) }
        assertFailsWith<IllegalArgumentException> {
            LibraryFoldersModel(folders = listOf(main, main.copy(id = LibraryFolderIdModel(value = "second"))))
        }
    }

    @Test
    fun `a location can only be used once`() {
        assertFailsWith<IllegalArgumentException> {
            folders.add(folder = share.copy(location = backup.location))
        }
        assertTrue(actual = folders.contains(location = backup.location))
        assertFalse(actual = folders.contains(location = share.location))
    }

    @Test
    fun `sync folders can be added and removed but the primary cannot be removed`() {
        val extended = folders.add(folder = share)

        assertEquals(expected = listOf(backup, share), actual = extended.syncFolders)
        assertEquals(expected = listOf(share), actual = extended.remove(id = backup.id).syncFolders)
        assertFailsWith<IllegalArgumentException> { extended.remove(id = main.id) }
    }

    @Test
    fun `choosing another primary demotes the old one`() {
        val switched = folders.withPrimary(id = backup.id)

        assertEquals(expected = backup.id, actual = switched.primary.id)
        assertEquals(expected = listOf(main.id), actual = switched.syncFolders.map { it.id })
    }

    private fun folder(id: String, role: FolderRole): LibraryFolderModel {
        return LibraryFolderModel(
            id = LibraryFolderIdModel(value = id),
            displayName = id,
            location = FolderLocationModel(value = "/tmp/$id"),
            role = role,
        )
    }
}
