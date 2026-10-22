package dms.domain.task

import dms.domain.catalog.CatalogMergeConflict
import dms.domain.change.DetectedChange
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.document.model.ContentHashModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.field.model.FieldKeyModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel
import dms.domain.rule.learn.model.RuleProposalModel
import dms.domain.task.model.TaskKeyModel
import java.time.LocalDate

/**
 * Something on the dashboard that waits for the user's decision. Nothing changes the library until
 * the user acts on a task.
 */
sealed interface OpenTask {

    val key: TaskKeyModel

    /** Approve, edit or reject what the AI suggested for a document. */
    data class ReviewSuggestion(val suggestion: ClassificationSuggestionModel) : OpenTask {
        override val key: TaskKeyModel
            get() {
                return TaskKeyModel(value = "suggestion:${suggestion.documentId}")
            }
    }

    /** Add an uncatalogued file to the catalog (and let the AI suggest a classification). */
    data class AdoptNewFile(val change: DetectedChange.NewFile) : OpenTask {
        override val key: TaskKeyModel
            get() {
                return TaskKeyModel(value = "new:${change.path}")
            }
    }

    /** Accept new content of a file, or restore the catalogued content from another folder. */
    data class ReviewModification(val change: DetectedChange.ModifiedFile) : OpenTask {
        override val key: TaskKeyModel
            get() {
                return TaskKeyModel(value = "modified:${change.documentId}:${change.folderId}")
            }
    }

    /** Apply a move made outside the tool to the catalog and the other folders, or undo it. */
    data class ConfirmMove(val change: DetectedChange.MovedFile) : OpenTask {
        override val key: TaskKeyModel
            get() {
                return TaskKeyModel(value = "moved:${change.documentId}:${change.folderId}")
            }
    }

    /** Restore a missing file from a folder that has it, or delete it everywhere. */
    data class RestoreOrDelete(val change: DetectedChange.MissingFile) : OpenTask {
        override val key: TaskKeyModel
            get() {
                return TaskKeyModel(value = "missing:${change.documentId}")
            }
    }

    /** Decide which content of a file to keep when the folders disagree. */
    data class ResolveContentConflict(val change: DetectedChange.ContentConflict) : OpenTask {
        override val key: TaskKeyModel
            get() {
                return TaskKeyModel(value = "conflict:${change.path}")
            }
    }

    /**
     * Keep one of several catalogued documents with the same size and content hash, and move the other
     * copies to the trash. [documentIds] has at least two entries.
     */
    data class ResolveDuplicates(val contentHash: ContentHashModel, val documentIds: List<DocumentIdModel>) : OpenTask {
        override val key: TaskKeyModel
            get() {
                return TaskKeyModel(value = "duplicates:$contentHash")
            }
    }

    /** Review a decision the catalog merge made between two catalog copies. */
    data class ReviewCatalogMerge(val conflict: CatalogMergeConflict) : OpenTask {
        override val key: TaskKeyModel
            get() {
                val subject = when (conflict) {
                    is CatalogMergeConflict.DocumentDiffers -> "document:${conflict.kept.id}"
                    is CatalogMergeConflict.PathClash -> "path:${conflict.discarded.id}"
                    is CatalogMergeConflict.CategoryDiffers -> "category:${conflict.kept.id}"
                    is CatalogMergeConflict.FieldDefinitionDiffers -> "field:${conflict.kept.key}"
                    is CatalogMergeConflict.SavedSearchDiffers -> "search:${conflict.kept.id}"
                    is CatalogMergeConflict.RuleDiffers -> "rule:${conflict.kept.id}"
                }
                return TaskKeyModel(value = "merge:$subject")
            }
    }

    /** Retry operations that could not be applied to a folder, for example while it was offline. */
    data class RetryFailedOperations(val folderId: LibraryFolderIdModel, val count: Int) : OpenTask {
        override val key: TaskKeyModel
            get() {
                return TaskKeyModel(value = "retry:$folderId")
            }
    }

    /**
     * A deadline field of a document (due date, cancel by, …) falls on [date], within its lead time.
     * The key holds the date, so a dismissed task returns when the date changes.
     */
    data class DeadlineDue(
        val documentId: DocumentIdModel,
        val fieldKey: FieldKeyModel,
        val date: LocalDate,
    ) : OpenTask {
        override val key: TaskKeyModel
            get() {
                return TaskKeyModel(value = "deadline:$documentId:$fieldKey:$date")
            }
    }

    /**
     * The retention the user set for a document ended on [date]: delete it, or mark it as done so it
     * stays an ordinary document.
     */
    data class RetentionDue(val documentId: DocumentIdModel, val date: LocalDate) : OpenTask {
        override val key: TaskKeyModel
            get() {
                return TaskKeyModel(value = "retention:$documentId:$date")
            }
    }

    /** [folder] holds [fileCount] files without any structure; the restructure assistant could sort them. */
    data class RestructureSuggested(val folder: RelativePathModel, val fileCount: Int) : OpenTask {
        override val key: TaskKeyModel
            get() {
                return TaskKeyModel(value = "restructure:$folder:$fileCount")
            }
    }

    /** Accept or reject a rule learned from the documents the user approved. */
    data class ReviewLearnedRule(val proposal: RuleProposalModel) : OpenTask {
        override val key: TaskKeyModel
            get() {
                return TaskKeyModel(value = "rule:${proposal.signature}")
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
