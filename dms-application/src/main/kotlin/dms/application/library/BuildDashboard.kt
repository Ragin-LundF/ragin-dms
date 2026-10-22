package dms.application.library

import dms.application.library.model.DashboardModel
import dms.application.library.model.FolderStateModel
import dms.application.library.model.ScanResultModel
import dms.application.port.state.ActivityLog
import dms.application.port.state.TaskStore
import dms.application.port.storage.OperationJournal
import dms.application.port.system.Clock
import dms.domain.change.DetectedChange
import dms.domain.deadline.DeadlinePlanner
import dms.domain.restructure.FolderShape
import dms.domain.rule.learn.RuleLearner
import dms.domain.task.OpenTask

/**
 * Collects everything that waits for the user: pending suggestions, the changes of the last scan,
 * catalogued documents that are copies of each other, decisions of the last catalog merge, failed
 * operations, flat folders the restructure assistant could sort, deadlines within their lead time
 * and ended retentions, and rules learned from approved documents — minus the tasks the user
 * dismissed, so a rejected rule proposal is not proposed again. New files in the inbox are no tasks:
 * the inbox lists them.
 */
class BuildDashboard(
    private val store: CatalogStore,
    private val tasks: TaskStore,
    private val journal: OperationJournal,
    private val activity: ActivityLog,
    private val clock: Clock,
) {

    /** Returns `null` while no library is open. [scan] is the last scan, if there was one. */
    suspend operator fun invoke(scan: ScanResultModel?): DashboardModel? {
        val library = store.current() ?: return null
        val pending = journal.pending().groupingBy { it.folderId }.eachCount()
        val documents = library.catalog.documents
        val openTasks = buildList {
            tasks.suggestions()
                .filter { it.documentId in documents }
                .forEach { add(OpenTask.ReviewSuggestion(suggestion = it)) }
            val inbox = library.catalog.settings.layout.inbox
            scan?.changes
                ?.filterNot { it is DetectedChange.NewFile && inbox.contains(candidate = it.path) }
                ?.forEach { add(OpenTask.forChange(change = it)) }
            documents.values.groupBy { it.contentHash to it.size }
                .filterValues { it.size > 1 }
                .forEach { (key, copies) ->
                    add(OpenTask.ResolveDuplicates(contentHash = key.first, documentIds = copies.map { it.id }))
                }
            tasks.mergeConflicts().forEach { add(OpenTask.ReviewCatalogMerge(conflict = it)) }
            pending.forEach { (folderId, count) ->
                add(OpenTask.RetryFailedOperations(folderId = folderId, count = count))
            }
            FolderShape.analyse(catalog = library.catalog, inbox = inbox).forEach {
                add(OpenTask.RestructureSuggested(folder = it.path, fileCount = it.fileCount))
            }
            addAll(elements = DeadlinePlanner.dueTasks(catalog = library.catalog, today = clock.today()))
            RuleLearner.proposals(catalog = library.catalog).forEach { add(OpenTask.ReviewLearnedRule(proposal = it)) }
        }
        val dismissed = tasks.dismissedKeys()
        return DashboardModel(
            tasks = openTasks.filterNot { it.key in dismissed },
            folders = library.folders.folders.map { folder ->
                FolderStateModel(
                    folder = folder,
                    reachable = scan?.let { folder.id in it.scannedFolders },
                    pendingOperations = pending[folder.id] ?: 0,
                )
            },
            documentCount = documents.size,
            uncategorizedCount = documents.values.count { it.categoryId == null },
            recentActivity = activity.recent(limit = RECENT_ACTIVITY),
            upcoming = DeadlinePlanner.upcoming(catalog = library.catalog, today = clock.today(), limit = UPCOMING),
        )
    }

    private companion object {
        const val RECENT_ACTIVITY = 20
        const val UPCOMING = 10
    }
}
