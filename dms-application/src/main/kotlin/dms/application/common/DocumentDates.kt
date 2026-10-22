package dms.application.common

import dms.application.port.storage.DocumentStorage
import dms.domain.document.FileNameDates
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel
import java.time.LocalDate

/**
 * The date of a document the user has not given one: the date in its file name, otherwise the day
 * the file was created. Without either the document has no date, and the day it was added stands in
 * for it wherever a date is shown. The user can change it at any time.
 */
class DocumentDates(private val storage: DocumentStorage) {

    suspend fun guess(folder: LibraryFolderModel, path: RelativePathModel): LocalDate? {
        return FileNameDates.dateIn(name = path.fileName) ?: storage.createdOn(folder = folder, path = path)
    }

    /** Metadata with the guessed date, for a document that is about to be catalogued. */
    suspend fun metadataFor(folder: LibraryFolderModel, path: RelativePathModel): DocumentMetadataModel {
        return DocumentMetadataModel(documentDate = guess(folder = folder, path = path))
    }
}
