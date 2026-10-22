package dms.storage.index

import dms.application.port.index.model.DocumentVectorsModel
import dms.application.port.index.model.IndexedChunkModel
import dms.domain.document.model.DocumentIdModel
import dms.storage.support.StorageResults
import dms.storage.support.TestFolders
import kotlinx.coroutines.test.runTest
import java.nio.file.Files
import java.nio.file.Path
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull

class FileVectorIndexTest {

    private val temp: Path = Files.createTempDirectory("dms-index-")
    private val folder = TestFolders.folder(location = temp)
    private val index = FileVectorIndex()
    private val documentId = DocumentIdModel(value = "doc-1")

    @AfterTest
    fun deleteTemp() {
        temp.toFile().deleteRecursively()
    }

    @Test
    fun `an entry is written to the hidden index folder and read back`() {
        runTest {
            val vectors = DocumentVectorsModel(
                documentId = documentId,
                model = "sample-embed",
                dimensions = 3,
                contentHash = TestFolders.hashOf(content = "alpha"),
                metadataHash = "1a2b",
                chunks = listOf(
                    IndexedChunkModel(start = 0, end = 10, vector = floatArrayOf(0.1f, 0.2f, 0.3f)),
                    IndexedChunkModel(start = 8, end = 20, vector = floatArrayOf(-1f, 0f, 1f)),
                ),
            )

            StorageResults.valueOf(result = index.write(folder = folder, vectors = vectors))
            val read = assertNotNull(actual = index.read(folder = folder, documentId = documentId))

            assertEquals(expected = true, actual = Files.isRegularFile(temp.resolve(".dms-index/doc-1.vec")))
            assertEquals(expected = "sample-embed", actual = read.model)
            assertEquals(expected = vectors.contentHash, actual = read.contentHash)
            assertEquals(expected = "1a2b", actual = read.metadataHash)
            assertEquals(expected = listOf(0 to 10, 8 to 20), actual = read.chunks.map { it.start to it.end })
            assertContentEquals(expected = floatArrayOf(-1f, 0f, 1f), actual = read.chunks[1].vector)
            assertEquals(
                expected = true,
                actual = read.isCurrent(
                    model = "sample-embed",
                    contentHash = vectors.contentHash,
                    metadataHash = "1a2b",
                ),
            )
            assertEquals(
                expected = false,
                actual = read.isCurrent(model = "other", contentHash = vectors.contentHash, metadataHash = "1a2b"),
            )
        }
    }

    @Test
    fun `a missing or broken entry reads as none`() {
        runTest {
            TestFolders.write(file = temp.resolve(".dms-index/doc-1.vec"), content = "not an index")

            assertNull(actual = index.read(folder = folder, documentId = documentId))
            assertNull(actual = index.read(folder = folder, documentId = DocumentIdModel(value = "other")))
        }
    }
}
