package dms.application.port.ai

import dms.application.port.ai.model.DocumentContent
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.LibraryFolderModel

/** Turns a document into text or page images for the classifier. Never modifies the file. */
interface DocumentContentReader {

    suspend fun read(folder: LibraryFolderModel, document: DocumentModel): DocumentContent

    /**
     * The whole text of a PDF's text layer or of an Office file, for search; `null` for images, scans
     * and files without text. Never renders pages, so it stays cheap.
     */
    suspend fun text(folder: LibraryFolderModel, document: DocumentModel): String?
}
