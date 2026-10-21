package dms.application.port.ai

import dms.application.port.ai.model.DocumentContent
import dms.domain.document.Document
import dms.domain.library.LibraryFolder

/** Turns a document into text or page images for the classifier. Never modifies the file. */
interface DocumentContentReader {

    suspend fun read(folder: LibraryFolder, document: Document): DocumentContent
}
