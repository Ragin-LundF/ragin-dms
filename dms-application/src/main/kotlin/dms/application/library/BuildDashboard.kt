package dms.application.library

import dms.application.library.model.Dashboard
import dms.application.library.model.FolderState
import dms.application.library.model.ScanResult
import dms.application.port.state.ActivityLog
import dms.application.port.state.TaskStore
import dms.application.port.storage.OperationJournal
import dms.domain.task.OpenTask

/**
 * Collects everything that waits for the user: pending suggestions, the changes of the last scan,
 * catalogued documents that are copies of each other, decisions of the last catalog merge and failed
 * operations — minus the tasks the user dismissed.
 */
class BuildDashboard(
    private val store: CatalogStore,
    private val tasks: TaskStore,
    private val journal: OperationJournal,
    private val activity: ActivityLog,
) {

    /** Returns `null` while no library is open. [scan] is the last scan, if there was one. */
    suspend operator fun invoke(scan: ScanResult?): Dashboard? {
        val library = store.current() ?: return null
        val pending = journal.pending().groupingBy { it.folderId }.eachCount()
        val documents = library.catalog.documents
        val openTasks = buildList {
            tasks.suggestions()
                .filter { it.documentId in documents }
                .forEach { add(OpenTask.ReviewSuggestion(suggestion = it)) }
            scan?.changes?.forEach { add(OpenTask.forChange(change = it)) }
            documents.values.groupBy { it.contentHash to it.size }
                .filterValues { it.size > 1 }
                .forEach { (key, copies) ->
                    add(OpenTask.ResolveDuplicates(contentHash = key.first, documentIds = copies.map { it.id }))
                }
            tasks.mergeConflicts().forEach { add(OpenTask.ReviewCatalogMerge(conflict = it)) }
            pending.forEach { (folderId, count) ->
                add(OpenTask.RetryFailedOperations(folderId = folderId, count = count))
            }
        }
        val dismissed = tasks.dismissedKeys()
        return Dashboard(
            tasks = openTasks.filterNot { it.key in dismissed },
            folders = library.folders.folders.map { folder ->
                FolderState(
                    folder = folder,
                    reachable = scan?.let { folder.id in it.scannedFolders },
                    pendingOperations = pending[folder.id] ?: 0,
                )
            },
            documentCount = documents.size,
            uncategorizedCount = documents.values.count { it.categoryId == null },
            recentActivity = activity.recent(limit = RECENT_ACTIVITY),
        )
    }

    private companion object {
        const val RECENT_ACTIVITY = 20
    }
}
