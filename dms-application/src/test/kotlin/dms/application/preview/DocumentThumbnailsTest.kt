package dms.application.preview

import dms.application.TestLibrary
import dms.application.fake.ai.FakePreviewer
import dms.application.port.ai.model.PageImageModel
import dms.application.preview.model.ThumbnailOutcome
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertSame

class DocumentThumbnailsTest {

    private val library = TestLibrary()
    private val previewer = FakePreviewer()
    private val thumbnails = DocumentThumbnails(store = library.store, previewer = previewer)
    private val scan = RelativePathModel.of(raw = "letters/scan.pdf")
    private val picture = PageImageModel(bytes = byteArrayOf(1, 2, 3), mimeType = "image/png")

    @Test
    fun `renders the picture of a file in a library folder at thumbnail size`() {
        runTest {
            library.open()
            previewer.pictures[scan] = picture

            val outcome = thumbnails(folderId = library.syncA.id, path = scan)

            assertSame(expected = picture, actual = assertIs<ThumbnailOutcome.Rendered>(value = outcome).image)
            assertEquals(
                expected = listOf(FakePreviewer.Request(folder = library.syncA, path = scan, maxSide = 320)),
                actual = previewer.requests,
            )
        }
    }

    @Test
    fun `a file without a picture has none`() {
        runTest {
            library.open()

            val outcome = thumbnails(folderId = library.primary.id, path = scan)

            assertEquals(expected = ThumbnailOutcome.NoPicture, actual = outcome)
        }
    }

    @Test
    fun `nothing is rendered while no library is open`() {
        runTest {
            previewer.pictures[scan] = picture

            val outcome = thumbnails(folderId = library.primary.id, path = scan)

            assertEquals(expected = ThumbnailOutcome.FolderNotFound, actual = outcome)
            assertEquals(expected = emptyList(), actual = previewer.requests)
        }
    }

    @Test
    fun `a folder that is not part of the library is not found`() {
        runTest {
            library.open()
            previewer.pictures[scan] = picture

            val outcome = thumbnails(folderId = LibraryFolderIdModel(value = "elsewhere"), path = scan)

            assertEquals(expected = ThumbnailOutcome.FolderNotFound, actual = outcome)
            assertEquals(expected = emptyList(), actual = previewer.requests)
        }
    }

    @Test
    fun `a file drawn again comes from memory`() {
        runTest {
            library.open()
            previewer.pictures[scan] = picture

            thumbnails(folderId = library.primary.id, path = scan)
            val again = thumbnails(folderId = library.primary.id, path = scan)

            assertSame(expected = picture, actual = assertIs<ThumbnailOutcome.Rendered>(value = again).image)
            assertEquals(expected = 1, actual = previewer.requests.size)
        }
    }

    @Test
    fun `a file without a picture is remembered as such`() {
        runTest {
            library.open()

            thumbnails(folderId = library.primary.id, path = scan)
            val again = thumbnails(folderId = library.primary.id, path = scan)

            assertEquals(expected = ThumbnailOutcome.NoPicture, actual = again)
            assertEquals(expected = 1, actual = previewer.requests.size)
        }
    }

    @Test
    fun `the same path in another folder is rendered on its own`() {
        runTest {
            library.open()
            previewer.pictures[scan] = picture

            thumbnails(folderId = library.primary.id, path = scan)
            thumbnails(folderId = library.syncB.id, path = scan)

            assertEquals(
                expected = listOf(library.primary, library.syncB),
                actual = previewer.requests.map { it.folder },
            )
        }
    }

    @Test
    fun `the least recently drawn file is forgotten once the memory is full`() {
        runTest {
            library.open()
            val paths = (0..CAPACITY).map { RelativePathModel.of(raw = "scans/page-$it.pdf") }

            paths.forEach { thumbnails(folderId = library.primary.id, path = it) }
            previewer.requests.clear()
            thumbnails(folderId = library.primary.id, path = paths.last())
            thumbnails(folderId = library.primary.id, path = paths.first())

            assertEquals(expected = listOf(paths.first()), actual = previewer.requests.map { it.path })
        }
    }

    @Test
    fun `drawing a file again keeps it in memory longer than files drawn later`() {
        runTest {
            library.open()
            val paths = (0..CAPACITY).map { RelativePathModel.of(raw = "scans/page-$it.pdf") }

            paths.dropLast(n = 1).forEach { thumbnails(folderId = library.primary.id, path = it) }
            thumbnails(folderId = library.primary.id, path = paths.first())
            thumbnails(folderId = library.primary.id, path = paths.last())
            previewer.requests.clear()
            thumbnails(folderId = library.primary.id, path = paths.first())
            thumbnails(folderId = library.primary.id, path = paths[1])

            assertEquals(expected = listOf(paths[1]), actual = previewer.requests.map { it.path })
        }
    }

    private companion object {
        const val CAPACITY = 200
    }
}
