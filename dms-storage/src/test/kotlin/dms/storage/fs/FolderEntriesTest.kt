package dms.storage.fs

import dms.application.port.storage.model.FolderEntriesModel
import dms.application.port.storage.model.StorageFailure
import dms.storage.support.RecordingTrash
import dms.storage.support.StorageResults
import dms.storage.support.TestFolders
import kotlinx.coroutines.test.runTest
import java.nio.file.Files
import java.nio.file.Path
import java.text.Normalizer
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals

class FolderEntriesTest {

    private val temp: Path = Files.createTempDirectory("dms-storage-")
    private val root: Path = Files.createDirectories(temp.resolve("library"))
    private val folder = TestFolders.folder(location = root)
    private val storage = FileSystemDocumentStorage(trash = RecordingTrash(directory = temp.resolve("trash")))

    @AfterTest
    fun deleteTemp() {
        temp.toFile().deleteRecursively()
    }

    @Test
    fun `every folder, also an empty one, and every file below a folder are listed, the tool's files left out`() {
        runTest {
            TestFolders.write(file = root.resolve("Home/Energy/power.pdf"), content = "power")
            Files.createDirectories(root.resolve("Home/Empty"))
            TestFolders.write(file = root.resolve("Home/.DS_Store"), content = "x")
            TestFolders.write(file = root.resolve("Home/Energy/.dms-tmp-1234"), content = "x")
            TestFolders.write(file = root.resolve("Other/letter.pdf"), content = "letter")

            val listed = storage.listEntries(folder = folder, path = TestFolders.path(raw = "Home"))
            val entries = StorageResults.valueOf(result = listed)

            assertEquals(
                expected = FolderEntriesModel(
                    folders = setOf(TestFolders.path(raw = "Home/Energy"), TestFolders.path(raw = "Home/Empty")),
                    files = setOf(TestFolders.path(raw = "Home/Energy/power.pdf")),
                ),
                actual = entries,
            )
        }
    }

    @Test
    fun `names come back composed, and a missing folder is not found`() {
        runTest {
            val decomposed = Normalizer.normalize("Bücher", Normalizer.Form.NFD)
            TestFolders.write(file = root.resolve(decomposed).resolve("a.pdf"), content = "a")

            val listed = storage.listEntries(folder = folder, path = TestFolders.path(raw = ""))
            val all = StorageResults.valueOf(result = listed)
            val missing = storage.listEntries(folder = folder, path = TestFolders.path(raw = "Nowhere"))

            assertEquals(expected = setOf(TestFolders.path(raw = "Bücher")), actual = all.folders)
            assertEquals(expected = StorageFailure.NOT_FOUND, actual = StorageResults.failureOf(result = missing))
        }
    }

    @Test
    fun `listing the folders finds every folder at any depth, also empty ones, and no files`() {
        runTest {
            TestFolders.write(file = root.resolve("Home/Energy/Old/gas.pdf"), content = "gas")
            Files.createDirectories(root.resolve("Home/Empty"))
            Files.createDirectories(root.resolve(".Trashes/kept"))
            TestFolders.write(file = root.resolve("letter.pdf"), content = "letter")

            val listed = StorageResults.valueOf(result = storage.listFolders(folder = folder))

            assertEquals(
                expected = setOf("Home", "Home/Energy", "Home/Energy/Old", "Home/Empty").map {
                    TestFolders.path(raw = it)
                }.toSet(),
                actual = listed,
            )
        }
    }
}
