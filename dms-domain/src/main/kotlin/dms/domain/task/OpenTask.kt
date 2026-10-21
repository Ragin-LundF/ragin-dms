package dms.domain.task

import dms.domain.catalog.CatalogMergeConflict
import dms.domain.change.DetectedChange
import dms.domain.classification.ClassificationSuggestion
import dms.domain.document.ContentHash
import dms.domain.document.DocumentId
import dms.domain.library.LibraryFolderId

/**
 * Something on the dashboard that waits for the user's decision. Nothing changes the library until
 * the user acts on a task.
 */
sealed interface OpenTask {

    val key: TaskKey

    /** Approve, edit or reject what the AI suggested for a document. */
    data class ReviewSuggestion(val suggestion: ClassificationSuggestion) : OpenTask {
        override val key: TaskKey
            get() {
                return TaskKey(value = "suggestion:${suggestion.documentId}")
            }
    }

    /** Add an uncatalogued file to the catalog (and let the AI suggest a classification). */
    data class AdoptNewFile(val change: DetectedChange.NewFile) : OpenTask {
        override val key: TaskKey
            get() {
                return TaskKey(value = "new:${change.path}")
            }
    }

    /** Accept new content of a file, or restore the catalogued content from another folder. */
    data class ReviewModification(val change: DetectedChange.ModifiedFile) : OpenTask {
        override val key: TaskKey
            get() {
                return TaskKey(value = "modified:${change.documentId}:${change.folderId}")
            }
    }

    /** Apply a move made outside the tool to the catalog and the other folders, or undo it. */
    data class ConfirmMove(val change: DetectedChange.MovedFile) : OpenTask {
        override val key: TaskKey
            get() {
                return TaskKey(value = "moved:${change.documentId}:${change.folderId}")
            }
    }

    /** Restore a missing file from a folder that has it, or delete it everywhere. */
    data class RestoreOrDelete(val change: DetectedChange.MissingFile) : OpenTask {
        override val key: TaskKey
            get() {
                return TaskKey(value = "missing:${change.documentId}")
            }
    }

    /** Decide which content of a file to keep when the folders disagree. */
    data class ResolveContentConflict(val change: DetectedChange.ContentConflict) : OpenTask {
        override val key: TaskKey
            get() {
                return TaskKey(value = "conflict:${change.path}")
            }
    }

    /**
     * Keep one of several catalogued documents with the same size and content hash, and move the other
     * copies to the trash. [documentIds] has at least two entries.
     */
    data class ResolveDuplicates(val contentHash: ContentHash, val documentIds: List<DocumentId>) : OpenTask {
        override val key: TaskKey
            get() {
                return TaskKey(value = "duplicates:$contentHash")
            }
    }

    /** Review a decision the catalog merge made between two catalog copies. */
    data class ReviewCatalogMerge(val conflict: CatalogMergeConflict) : OpenTask {
        override val key: TaskKey
            get() {
                val subject = when (conflict) {
                    is CatalogMergeConflict.DocumentDiffers -> "document:${conflict.kept.id}"
                    is CatalogMergeConflict.PathClash -> "path:${conflict.discarded.id}"
                    is CatalogMergeConflict.CategoryDiffers -> "category:${conflict.kept.id}"
                }
                return TaskKey(value = "merge:$subject")
            }
    }

    /** Retry operations that could not be applied to a folder, for example while it was offline. */
    data class RetryFailedOperations(val folderId: LibraryFolderId, val count: Int) : OpenTask {
        override val key: TaskKey
            get() {
                return TaskKey(value = "retry:$folderId")
            }
    }

    companion object {

        /** The task that asks the user what to do about [change]. */
        fun forChange(change: DetectedChange): OpenTask {
            return when (change) {
                is DetectedChange.NewFile -> AdoptNewFile(change = change)
                is DetectedChange.ModifiedFile -> ReviewModification(change = change)
                is DetectedChange.MovedFile -> ConfirmMove(change = change)
                is DetectedChange.MissingFile -> RestoreOrDelete(change = change)
                is DetectedChange.ContentConflict -> ResolveContentConflict(change = change)
            }
        }
    }
}
