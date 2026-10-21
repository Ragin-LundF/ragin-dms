package dms.application.port.ai

import dms.application.port.ai.model.DocumentContent
import dms.domain.document.Document
import dms.domain.library.LibraryFolder

/** Turns a document into text or page images for the classifier. Never modifies the file. */
interface DocumentContentReader {

    suspend fun read(folder: LibraryFolder, document: Document): DocumentContent

    /**
     * The whole text of a PDF's text layer or of an Office file, for search; `null` for images, scans
     * and files without text. Never renders pages, so it stays cheap.
     */
    suspend fun text(folder: LibraryFolder, document: Document): String?
}
