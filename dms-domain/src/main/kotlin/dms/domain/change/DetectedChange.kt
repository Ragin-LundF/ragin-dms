package dms.domain.change

import dms.domain.document.ContentHash
import dms.domain.document.DocumentId
import dms.domain.library.LibraryFolderId
import dms.domain.library.RelativePath

/** A difference between the catalog and the library folders, found by [ChangeDetector]. */
sealed interface DetectedChange {

    val path: RelativePath

    /** A file the catalog does not know, present in the listed folders with the same content. */
    data class NewFile(
        override val path: RelativePath,
        val presentIn: Set<LibraryFolderId>,
        val size: Long,
        val contentHash: ContentHash?,
    ) : DetectedChange

    /** A catalogued file whose content differs from the catalog in one folder. */
    data class ModifiedFile(
        val documentId: DocumentId,
        override val path: RelativePath,
        val folderId: LibraryFolderId,
        val size: Long,
        val contentHash: ContentHash?,
    ) : DetectedChange

    /** A catalogued file found under another path in one folder, recognized by its content. */
    data class MovedFile(
        val documentId: DocumentId,
        override val path: RelativePath,
        val previousPath: RelativePath,
        val folderId: LibraryFolderId,
    ) : DetectedChange

    /**
     * A catalogued file missing from some folders. [presentIn] lists the folders that still have it;
     * when it is empty, the file is gone from every reachable folder.
     */
    data class MissingFile(
        val documentId: DocumentId,
        override val path: RelativePath,
        val missingIn: Set<LibraryFolderId>,
        val presentIn: Set<LibraryFolderId>,
    ) : DetectedChange

    /** A file the catalog does not know, with different content in different folders. */
    data class ContentConflict(
        override val path: RelativePath,
        val contentHashes: Map<LibraryFolderId, ContentHash>,
    ) : DetectedChange
}
