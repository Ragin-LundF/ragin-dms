package dms.ui.dashboard.task

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.OpenInNew
import androidx.compose.material.icons.rounded.AutoAwesome
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.Refresh
import androidx.compose.material.icons.rounded.Restore
import androidx.compose.material.icons.rounded.Sync
import dms.application.resolution.DuplicatePreference
import dms.application.resolution.model.MoveDecision
import dms.domain.change.DetectedChange
import dms.domain.document.Document
import dms.domain.library.LibraryFolderId
import dms.domain.library.RelativePath
import dms.domain.task.OpenTask
import dms.ui.components.model.ActionButton
import dms.ui.components.model.ButtonStyle
import dms.ui.dashboard.model.DuplicateCopy
import dms.ui.dashboard.model.TaskFile

/** The buttons of every kind of task: exactly the actions its use cases support. */
object TaskActions {

    fun buttonsFor(task: OpenTask, context: TaskContext, handler: TaskHandler): List<ActionButton> {
        val dismiss = ActionButton(label = "Dismiss", style = ButtonStyle.TEXT) { handler.dismiss(key = task.key) }
        return when (task) {
            is OpenTask.ReviewSuggestion -> listOf(
                ActionButton(label = "Review", icon = Icons.Rounded.AutoAwesome, style = ButtonStyle.TONAL) {
                    handler.review(suggestion = task.suggestion)
                },
                ActionButton(label = "Reject", style = ButtonStyle.TEXT) {
                    handler.reject(documentId = task.suggestion.documentId)
                },
            )

            is OpenTask.AdoptNewFile -> newFileButtons(change = task.change, context = context, handler = handler) +
                dismiss

            is OpenTask.RestoreOrDelete -> listOf(
                ActionButton(
                    label = "Restore",
                    icon = Icons.Rounded.Restore,
                    style = ButtonStyle.TONAL,
                    disabledReason = "No reachable folder still has this file"
                        .takeIf { task.change.presentIn.isEmpty() },
                ) { handler.restore(change = task.change) },
                ActionButton(label = "Move to trash…") { handler.deleteEverywhere(change = task.change) },
                dismiss,
            )

            is OpenTask.ReviewModification -> modificationButtons(task = task, context = context, handler = handler) +
                dismiss

            is OpenTask.ConfirmMove -> listOf(
                ActionButton(
                    label = "Rename to ${task.change.previousPath.fileName} here",
                    icon = Icons.Rounded.Sync,
                    style = ButtonStyle.TONAL,
                ) { handler.resolveMove(change = task.change, decision = MoveDecision.UNDO) },
                ActionButton(label = "Use ${task.change.path.fileName} everywhere") {
                    handler.resolveMove(change = task.change, decision = MoveDecision.APPLY)
                },
                dismiss,
            )

            is OpenTask.ResolveDuplicates -> listOf(dismiss)

            is OpenTask.ResolveContentConflict -> conflictButtons(task = task, context = context, handler = handler) +
                dismiss

            is OpenTask.ReviewCatalogMerge -> listOf(ActionButton(label = "OK") { handler.dismiss(key = task.key) })
            is OpenTask.RetryFailedOperations -> listOf(
                ActionButton(label = "Retry", icon = Icons.Rounded.Refresh, style = ButtonStyle.TONAL) {
                    handler.retry()
                },
                dismiss,
            )
        }
    }

    /** The files [task] is about, the primary's first; one per folder when their versions differ. */
    fun filesFor(task: OpenTask, context: TaskContext): List<TaskFile> {
        return when (task) {
            is OpenTask.ReviewSuggestion -> {
                val path = context.pathOf(id = task.suggestion.documentId)
                val primary = context.primaryId
                if (path == null || primary == null) {
                    emptyList()
                } else {
                    listOf(TaskFile(label = "Open", folderId = primary, path = path))
                }
            }

            is OpenTask.AdoptNewFile -> firstOf(
                folderIds = task.change.presentIn,
                path = task.change.path,
                context = context,
            )

            is OpenTask.RestoreOrDelete -> firstOf(
                folderIds = task.change.presentIn,
                path = task.change.path,
                context = context,
            )

            is OpenTask.ReviewModification -> modificationFiles(task = task, context = context)
            is OpenTask.ConfirmMove -> listOf(
                TaskFile(label = "Open", folderId = task.change.folderId, path = task.change.path),
            )

            is OpenTask.ResolveDuplicates -> duplicateFiles(task = task, context = context)

            is OpenTask.ResolveContentConflict -> context.primaryFirst(ids = task.change.contentHashes.keys).map {
                TaskFile(label = "Open ${context.folderName(id = it)} version", folderId = it, path = task.change.path)
            }

            is OpenTask.ReviewCatalogMerge -> emptyList()
            is OpenTask.RetryFailedOperations -> emptyList()
        }
    }

    /** A button for each of [files] that opens it in its default application. */
    fun opensOf(files: List<TaskFile>, handler: TaskHandler): List<ActionButton> {
        return files.map { openButton(file = it, handler = handler) }
    }

    private fun openButton(file: TaskFile, handler: TaskHandler): ActionButton {
        return ActionButton(label = file.label, icon = Icons.AutoMirrored.Rounded.OpenInNew, style = ButtonStyle.TEXT) {
            handler.open(folderId = file.folderId, path = file.path)
        }
    }

    private fun firstOf(
        folderIds: Collection<LibraryFolderId>,
        path: RelativePath,
        context: TaskContext,
    ): List<TaskFile> {
        return context.primaryFirst(ids = folderIds).take(n = 1).map {
            TaskFile(label = "Open", folderId = it, path = path)
        }
    }

    private fun duplicateFiles(task: OpenTask.ResolveDuplicates, context: TaskContext): List<TaskFile> {
        val primary = context.primaryId ?: return emptyList()
        return context.documents(ids = task.documentIds).map {
            TaskFile(label = "Open ${it.path.fileName}", folderId = primary, path = it.path)
        }
    }

    private fun modificationFiles(task: OpenTask.ReviewModification, context: TaskContext): List<TaskFile> {
        val change = task.change
        val changed = TaskFile(label = "Open new version", folderId = change.folderId, path = change.path)
        val previous = context.otherFolderThan(folderId = change.folderId)?.let {
            TaskFile(label = "Open previous version", folderId = it, path = change.path)
        }
        return listOfNotNull(changed, previous)
    }

    private fun newFileButtons(
        change: DetectedChange.NewFile,
        context: TaskContext,
        handler: TaskHandler,
    ): List<ActionButton> {
        if (context.originalOf(change = change) != null) {
            return listOf(
                ActionButton(label = "Move copy to trash…", icon = Icons.Rounded.Delete, style = ButtonStyle.TONAL) {
                    handler.trashDuplicateFile(change = change)
                },
                ActionButton(label = "Adopt anyway") { handler.adopt(change = change) },
            )
        }
        return listOf(
            ActionButton(
                label = "Adopt",
                icon = Icons.Rounded.Check,
                style = ButtonStyle.TONAL,
                disabledReason = "The file was not read yet; scan again".takeIf { change.contentHash == null },
            ) { handler.adopt(change = change) },
            ActionButton(label = "Move to trash…") { handler.trashNewFile(change = change) },
        )
    }

    /** The copies of a duplicates task, the one with the most assigned values first; empty for other tasks. */
    fun copiesFor(task: OpenTask, context: TaskContext, handler: TaskHandler): List<DuplicateCopy> {
        val duplicates = task as? OpenTask.ResolveDuplicates ?: return emptyList()
        val copies = context.documents(ids = duplicates.documentIds)
        val primary = context.primaryId
        if (copies.size < 2 || primary == null) {
            return emptyList()
        }
        val preferred = DuplicatePreference.preferred(copies = copies)
        return copies.sortedBy { it.id != preferred.id }.map { copy ->
            val recommended = copy.id == preferred.id
            val values = assignedValues(document = copy, context = context)
            val file = TaskFile(label = "Open", folderId = primary, path = copy.path)
            DuplicateCopy(
                path = copy.path.value,
                detail = if (recommended) "Recommended · $values" else values,
                open = openButton(file = file, handler = handler),
                keep = ActionButton(
                    label = "Keep…",
                    style = if (recommended) ButtonStyle.TONAL else ButtonStyle.OUTLINED,
                ) { handler.keepDuplicate(task = duplicates, keep = copy.id) },
            )
        }
    }

    private fun assignedValues(document: Document, context: TaskContext): String {
        val values = listOfNotNull(
            document.categoryId?.let { "Category: ${context.categoryName(id = it)}" },
            document.tags.takeIf { it.isNotEmpty() }?.let { tags -> "Tags: ${tags.joinToString { it.value }}" },
            document.metadata.title?.let { "Title: $it" },
        )
        return values.joinToString(separator = " · ").ifEmpty { "Nothing assigned" }
    }

    private fun modificationButtons(
        task: OpenTask.ReviewModification,
        context: TaskContext,
        handler: TaskHandler,
    ): List<ActionButton> {
        val catalogued = context.otherFolderThan(folderId = task.change.folderId)
        return listOf(
            ActionButton(label = "Keep new content…") {
                handler.keepVersion(change = task.change, sourceFolderId = task.change.folderId)
            },
            ActionButton(
                label = "Restore previous…",
                disabledReason = "No other folder has the previous content".takeIf { catalogued == null },
            ) {
                catalogued?.let { handler.keepVersion(change = task.change, sourceFolderId = it) }
            },
        )
    }

    private fun conflictButtons(
        task: OpenTask.ResolveContentConflict,
        context: TaskContext,
        handler: TaskHandler,
    ): List<ActionButton> {
        val folderIds = task.change.contentHashes.keys.sortedBy { it != context.primaryId }
        val keep = folderIds.map { folderId ->
            ActionButton(label = "Keep ${context.folderName(id = folderId)}…") {
                handler.keepVersion(change = task.change, sourceFolderId = folderId)
            }
        }
        val renamed = folderIds.firstOrNull { it != context.primaryId }
        val keepBoth = ActionButton(
            label = "Keep both",
            disabledReason = "Only the primary folder has a version".takeIf { renamed == null },
        ) {
            renamed?.let { handler.keepBoth(change = task.change, renamedIn = it) }
        }
        return keep + keepBoth
    }
}
