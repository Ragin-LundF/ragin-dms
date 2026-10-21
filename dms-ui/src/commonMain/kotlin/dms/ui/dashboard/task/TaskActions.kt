package dms.ui.dashboard.task

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.AutoAwesome
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material.icons.rounded.Refresh
import androidx.compose.material.icons.rounded.Restore
import dms.application.resolution.model.MoveDecision
import dms.domain.task.OpenTask
import dms.ui.components.model.ActionButton
import dms.ui.components.model.ButtonStyle

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

            is OpenTask.AdoptNewFile -> listOf(
                ActionButton(
                    label = "Adopt",
                    icon = Icons.Rounded.Check,
                    style = ButtonStyle.TONAL,
                    disabledReason = "The file was not read yet; scan again".takeIf { task.change.contentHash == null },
                ) { handler.adopt(change = task.change) },
                dismiss,
            )

            is OpenTask.RestoreOrDelete -> listOf(
                ActionButton(
                    label = "Restore",
                    icon = Icons.Rounded.Restore,
                    style = ButtonStyle.TONAL,
                    disabledReason = "No reachable folder still has this file"
                        .takeIf { task.change.presentIn.isEmpty() },
                ) { handler.restore(change = task.change) },
                ActionButton(label = "Delete everywhere…") { handler.deleteEverywhere(change = task.change) },
                dismiss,
            )

            is OpenTask.ReviewModification -> modificationButtons(task = task, context = context, handler = handler) +
                dismiss

            is OpenTask.ConfirmMove -> listOf(
                ActionButton(label = "Apply move", icon = Icons.Rounded.Check, style = ButtonStyle.TONAL) {
                    handler.resolveMove(change = task.change, decision = MoveDecision.APPLY)
                },
                ActionButton(label = "Undo move") {
                    handler.resolveMove(change = task.change, decision = MoveDecision.UNDO)
                },
                dismiss,
            )

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
