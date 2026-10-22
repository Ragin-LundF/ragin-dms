package dms.ui.dashboard.bulk

import dms.application.common.OperationOutcome
import dms.application.resolution.DuplicatePreference
import dms.application.resolution.model.MoveDecision
import dms.domain.change.DetectedChange
import dms.domain.task.OpenTask
import dms.ui.components.dialog.DialogController
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ConfirmationRequestModel
import dms.ui.dashboard.model.TaskGroup
import dms.ui.dashboard.task.TaskContext
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.model.UiMessageModel

/**
 * The buttons that answer every task of a group at once, one task after the other, and what they do.
 * Removing duplicates asks first and lists every file that goes to the trash.
 */
class BulkActions(
    private val services: AppServices,
    private val session: LibrarySession,
    private val dialogs: DialogController,
) {

    fun buttonsFor(group: TaskGroup, tasks: List<OpenTask>, context: TaskContext): List<ActionButtonModel> {
        return when (group) {
            TaskGroup.NEW_FILES -> {
                // Of several new files with the same content only one is adopted; the others become duplicates.
                val readable = tasks.filterIsInstance<OpenTask.AdoptNewFile>().map { it.change }
                    .filter { it.contentHash != null }
                    .distinctBy { it.contentHash }
                listOf(ActionButtonModel(label = "Adopt all (${readable.size})") { adoptAll(changes = readable) })
            }

            TaskGroup.MISSING -> {
                val restorable = tasks.filterIsInstance<OpenTask.RestoreOrDelete>().map { it.change }
                    .filter { it.presentIn.isNotEmpty() }
                listOf(
                    ActionButtonModel(label = "Restore all (${restorable.size})") { restoreAll(changes = restorable) },
                )
            }

            TaskGroup.MOVED -> {
                val changes = tasks.filterIsInstance<OpenTask.ConfirmMove>().map { it.change }
                listOf(
                    ActionButtonModel(label = "Rename all to match (${changes.size})") { renameAll(changes = changes) },
                )
            }

            TaskGroup.DUPLICATES -> {
                val files = tasks.filterIsInstance<OpenTask.AdoptNewFile>().map { it.change }
                val catalogued = tasks.filterIsInstance<OpenTask.ResolveDuplicates>()
                val label = "Remove all duplicates (${files.size + catalogued.size})…"
                val removeAll = ActionButtonModel(label = label) {
                    removeAll(files = files, catalogued = catalogued, context = context)
                }
                listOf(removeAll)
            }

            else -> emptyList()
        }
    }

    /** Adopts every listed new file, one after the other; can be stopped between two files. */
    private fun adoptAll(changes: List<DetectedChange.NewFile>) {
        session.perform(label = "Adopting ${changes.size} new files", cancellable = true) {
            val adopted = session.forEachWithProgress(step = "Adopting", items = changes, name = { it.path.value }) {
                resolved(change = it, outcome = services.documents.adoptNewFile(change = it))
            }
            session.report(message = countMessage(verb = "Adopted", done = adopted, total = changes.size))
        }
    }

    /** Restores every listed missing file from a folder that still has it. */
    private fun restoreAll(changes: List<DetectedChange.MissingFile>) {
        session.perform(label = "Restoring ${changes.size} files") {
            val restored = session.forEachWithProgress(step = "Restoring", items = changes, name = { it.path.value }) {
                resolved(change = it, outcome = services.documents.restoreDocument(change = it))
            }
            session.report(message = countMessage(verb = "Restored", done = restored, total = changes.size))
        }
    }

    /** Renames each listed file in the folder where it was found to the name the catalog knows it by. */
    private fun renameAll(changes: List<DetectedChange.MovedFile>) {
        session.perform(label = "Renaming ${changes.size} files to match", cancellable = true) {
            val renamed = session.forEachWithProgress(step = "Renaming", items = changes, name = { it.path.value }) {
                val outcome = services.sync.resolveMove(change = it, decision = MoveDecision.UNDO)
                resolved(change = it, outcome = outcome)
            }
            session.report(message = countMessage(verb = "Renamed", done = renamed, total = changes.size))
        }
    }

    private fun removeAll(
        files: List<DetectedChange.NewFile>,
        catalogued: List<OpenTask.ResolveDuplicates>,
        context: TaskContext,
    ) {
        val affected = files.map { file ->
            "${file.path} (copy of ${context.originalOf(change = file)?.path ?: "a document"})"
        } + catalogued.flatMap { task ->
            val copies = context.documents(ids = task.documentIds)
            val kept = DuplicatePreference.preferred(copies = copies)
            copies.filter { it.id != kept.id }.map { "${it.path} (copy of ${kept.path})" }
        }
        dialogs.ask(
            request = ConfirmationRequestModel(
                title = "Remove all duplicates",
                message = "Each file has exactly the same size and SHA-256 checksum as the document named after it, " +
                    "and is compared with it byte by byte once more right before it goes to the trash, in every " +
                    "folder. Where a document keeps one of its copies, it takes over the copy's categories and tags.",
                affected = affected,
                confirmLabel = "Move duplicates to trash",
            ),
        ) {
            session.perform(label = "Removing duplicates", cancellable = true) {
                val trashed = session.forEachWithProgress(
                    step = "Removing copy",
                    items = files,
                    name = { it.path.value },
                ) { file ->
                    val outcome = services.documents.trashDuplicateFile(change = file, confirmed = true)
                    resolved(change = file, outcome = outcome)
                }
                val merged = session.forEachWithProgress(
                    step = "Removing copies",
                    items = catalogued,
                    name = { context.documentPath(id = it.documentIds.first()) },
                ) { task -> merge(task = task) }
                val total = files.size + catalogued.size
                val done = trashed + merged
                session.report(message = countMessage(verb = "Removed duplicates of", done = done, total = total))
            }
        }
    }

    /** Keeps the preferred copy of [task] in the current catalog and trashes the others. */
    private suspend fun merge(task: OpenTask.ResolveDuplicates): Boolean {
        val documents = services.library.catalog.value?.documents.orEmpty()
        val copies = task.documentIds.mapNotNull { documents[it] }
        if (copies.size < 2) {
            return false
        }
        val keep = DuplicatePreference.preferred(copies = copies).id
        return services.sync.mergeDuplicates(keep = keep, confirmed = true) is OperationOutcome.Done
    }

    /** Forgets [change] when [outcome] resolved it; returns whether it did. */
    private fun resolved(change: DetectedChange, outcome: OperationOutcome): Boolean {
        val done = outcome is OperationOutcome.Done
        if (done) {
            session.forget(change = change)
        }
        return done
    }

    private fun countMessage(verb: String, done: Int, total: Int): UiMessageModel {
        return UiMessageModel(text = "$verb $done of $total files", isError = done < total)
    }
}
