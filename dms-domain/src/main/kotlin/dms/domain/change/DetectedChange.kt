package dms.domain.change

import dms.domain.document.model.ContentHashModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel

/** A difference between the catalog and the library folders, found by [ChangeDetector]. */
sealed interface DetectedChange {

    val path: RelativePathModel

    /** A file the catalog does not know, present in the listed folders with the same content. */
    data class NewFile(
        override val path: RelativePathModel,
        val presentIn: Set<LibraryFolderIdModel>,
        val size: Long,
        val contentHash: ContentHashModel?,
    ) : DetectedChange

    /** A catalogued file whose content differs from the catalog in one folder. */
    data class ModifiedFile(
        val documentId: DocumentIdModel,
        override val path: RelativePathModel,
        val folderId: LibraryFolderIdModel,
        val size: Long,
        val contentHash: ContentHashModel?,
    ) : DetectedChange

    /** A catalogued file found under another path in one folder, recognized by its content. */
    data class MovedFile(
        val documentId: DocumentIdModel,
        override val path: RelativePathModel,
        val previousPath: RelativePathModel,
        val folderId: LibraryFolderIdModel,
    ) : DetectedChange

    /**
     * A catalogued file missing from some folders. [presentIn] lists the folders that still have it;
     * when it is empty, the file is gone from every reachable folder.
     */
    data class MissingFile(
        val documentId: DocumentIdModel,
        override val path: RelativePathModel,
        val missingIn: Set<LibraryFolderIdModel>,
        val presentIn: Set<LibraryFolderIdModel>,
    ) : DetectedChange

    /** A file the catalog does not know, with different content in different folders. */
    data class ContentConflict(
        override val path: RelativePathModel,
        val contentHashes: Map<LibraryFolderIdModel, ContentHashModel>,
    ) : DetectedChange
}
