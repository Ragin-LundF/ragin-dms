package dms.ai.read

import dms.application.port.ai.DocumentPreviewer
import dms.application.port.ai.model.PageImage
import dms.domain.document.DocumentKind
import dms.domain.library.LibraryFolder
import dms.domain.library.RelativePath
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.IOException
import java.nio.file.Path

/**
 * Renders thumbnails of PDFs and images in a library folder. Office files and other types have none,
 * and neither has a file that cannot be read. [locate] finds the file on disk.
 */
class FilePreviewer(private val locate: (LibraryFolder, RelativePath) -> Path?) : DocumentPreviewer {

    override suspend fun thumbnail(folder: LibraryFolder, path: RelativePath, maxSide: Int): PageImage? {
        return withContext(context = Dispatchers.IO) {
            val file = locate(folder, path) ?: return@withContext null
            try {
                when (DocumentKind.of(path = path)) {
                    DocumentKind.PDF -> PdfContent.thumbnail(file = file, maxSide = maxSide)
                    DocumentKind.IMAGE -> ImageContent.thumbnail(file = file, maxSide = maxSide)
                    DocumentKind.OFFICE, DocumentKind.OTHER -> null
                }
            } catch (ignored: IOException) {
                null
            } catch (ignored: IllegalArgumentException) {
                null
            }
        }
    }
}
