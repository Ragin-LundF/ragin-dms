package dms.ai.read

import dms.application.port.ai.DocumentContentReader
import dms.application.port.ai.model.DocumentContent
import dms.application.port.state.model.AiSettings
import dms.domain.document.Document
import dms.domain.document.DocumentKind
import dms.domain.library.LibraryFolder
import dms.domain.library.RelativePath
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.IOException
import java.nio.file.Path

/**
 * Reads a document from a library folder for the classifier and for search. Only reads; the file is
 * never changed. [locate] finds the file on disk, the way the storage adapters do.
 */
class FileContentReader(
    private val locate: (LibraryFolder, RelativePath) -> Path?,
    private val settings: suspend () -> AiSettings,
) : DocumentContentReader {

    override suspend fun read(folder: LibraryFolder, document: Document): DocumentContent {
        val preferVision = settings().preferVisionForPdfs
        return withContext(context = Dispatchers.IO) {
            val file = locate(folder, document.path) ?: return@withContext DocumentContent.NotReadable(
                reason = "The file is missing in ${folder.displayName}",
            )
            try {
                when (document.kind) {
                    DocumentKind.PDF -> PdfContent.read(file = file, preferVision = preferVision)
                    DocumentKind.IMAGE -> ImageContent.read(file = file)
                    DocumentKind.OFFICE -> OfficeContent.read(file = file)
                    DocumentKind.OTHER -> DocumentContent.NotReadable(reason = "This file type is not sent to the AI")
                }
            } catch (exception: IOException) {
                DocumentContent.NotReadable(reason = exception.message ?: "The file could not be read")
            } catch (exception: IllegalArgumentException) {
                // How POI reports a damaged or mislabeled Office file.
                DocumentContent.NotReadable(reason = exception.message ?: "The file format is not supported")
            }
        }
    }

    override suspend fun text(folder: LibraryFolder, document: Document): String? {
        return withContext(context = Dispatchers.IO) {
            val file = locate(folder, document.path) ?: return@withContext null
            try {
                when (document.kind) {
                    DocumentKind.PDF -> PdfContent.text(file = file)
                    DocumentKind.OFFICE -> OfficeContent.text(file = file)
                    DocumentKind.IMAGE, DocumentKind.OTHER -> null
                }?.takeIf { it.isNotEmpty() }
            } catch (ignored: IOException) {
                null
            } catch (ignored: IllegalArgumentException) {
                null
            }
        }
    }
}
