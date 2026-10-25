package dms.application.restructure

import dms.application.common.ActivityRecorder
import dms.application.common.OperationOutcome
import dms.application.common.model.LibrarySnapshotModel
import dms.application.library.CatalogStore
import dms.application.port.plan.PlanStore
import dms.application.port.plan.model.RestructureRecordModel
import dms.application.port.state.model.ActivityKind
import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.model.FolderOperation
import dms.application.port.storage.model.StorageResult
import dms.application.port.system.Clock
import dms.application.restructure.model.RestructureOutcome
import dms.application.sync.Replicator
import dms.application.sync.model.Replication
import dms.domain.catalog.CatalogUpdate
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel
import dms.domain.restructure.model.PlannedMoveModel
import dms.domain.restructure.model.RestructurePlanModel
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.withContext
import java.time.ZoneOffset
import java.time.format.DateTimeFormatter
import java.util.Locale
import kotlin.coroutines.cancellation.CancellationException

/**
 * Applies an approved restructure: moves the files one by one in the primary, journals each move for
 * the sync folders, and commits the catalog every [BATCH_SIZE] moves. Cancelling stops between two
 * files and still commits what was done, so the catalog always matches the disk. What was done is
 * recorded in `.dms-plans/` (replicated), so it can be reversed.
 *
 * Files are moved before the catalog is committed. A batch whose commit fails has still moved: it is
 * tried once more at the end, and what is still not catalogued is recorded and reported on its own.
 */
class ApplyRestructurePlan(
    private val store: CatalogStore,
    private val storage: DocumentStorage,
    private val replicator: Replicator,
    private val plans: PlanStore,
    private val activity: ActivityRecorder,
    private val clock: Clock,
) {

    suspend operator fun invoke(
        plan: RestructurePlanModel,
        skipped: Int = 0,
        onProgress: (done: Int, total: Int) -> Unit = { _, _ -> },
    ): RestructureOutcome {
        val library = store.current() ?: return RestructureOutcome.LibraryNotOpen
        val run = Run()
        val cancelled = try {
            plan.moves.forEachIndexed { index, move ->
                currentCoroutineContext().ensureActive()
                moveFile(library = library, move = move, run = run)
                if (run.batch.size >= BATCH_SIZE) {
                    commit(run = run)
                }
                onProgress(index + 1, plan.moves.size)
            }
            false
        } catch (cancellation: CancellationException) {
            withContext(context = NonCancellable) {
                finish(plan = plan, run = run, skipped = skipped, cancelled = true)
            }
            throw cancellation
        }
        return finish(plan = plan, run = run, skipped = skipped, cancelled = cancelled)
    }

    private suspend fun finish(
        plan: RestructurePlanModel,
        run: Run,
        skipped: Int,
        cancelled: Boolean,
    ): RestructureOutcome {
        commit(run = run)
        // One more try for the moves whose catalog commit failed; their files have moved already.
        run.batch += run.notCatalogued
        run.notCatalogued.clear()
        commit(run = run)
        val record = record(plan = plan, moves = run.done + run.notCatalogued, run = run)
        if (run.done.isNotEmpty()) {
            activity.record(
                kind = ActivityKind.RESTRUCTURED,
                subject = "${run.done.size} documents by ${plan.template}",
            )
        }
        return RestructureOutcome.Applied(
            moved = run.done.size,
            failed = run.failed,
            notCatalogued = run.notCatalogued.toList(),
            skipped = skipped,
            pendingFolders = run.pending,
            record = record,
            emptyFolders = emptyFolders(sources = run.done.mapNotNull { it.from.parent }.toSet()),
            cancelled = cancelled,
        )
    }

    /**
     * Moves one file in the primary and journals the move for the sync folders. A document the catalog
     * already has at the target only needs its file to follow; this is how a reversal brings back a file
     * whose move was never catalogued.
     */
    private suspend fun moveFile(library: LibrarySnapshotModel, move: PlannedMoveModel, run: Run) {
        val catalog = store.catalog.value ?: library.catalog
        val current = catalog.documents[move.documentId]?.path
        val ownerId = catalog.findByPath(path = move.to)?.id ?: move.documentId
        if (current !in setOf(move.from, move.to) || ownerId != move.documentId) {
            run.failed += move
            return
        }
        val replication = replicator.replicate(
            folders = library.folders,
            onPrimary = { primary -> storage.move(folder = primary, from = move.from, to = move.to) },
            followUp = FolderOperation.Move(from = move.from, to = move.to),
        )
        when (replication) {
            is Replication.PrimaryFailed -> run.failed += move
            is Replication.Done -> {
                run.batch += move
                run.pending += replication.pendingFolders
            }
        }
    }

    /** Commits the moves of the current batch in one catalog revision. */
    private suspend fun commit(run: Run) {
        if (run.batch.isEmpty()) {
            return
        }
        val batch = run.batch.toList()
        val outcome = store.commit { catalog ->
            val moves = batch.filter { catalog.documents[it.documentId]?.path == it.from }
            if (moves.isEmpty()) {
                CatalogUpdate.Applied(catalog = catalog)
            } else {
                catalog.moveDocuments(moves = moves.associate { it.documentId to it.to })
            }
        }
        if (outcome is OperationOutcome.Done) {
            run.done += batch
            run.pending += outcome.pendingFolders
        } else {
            run.notCatalogued += batch
        }
        run.batch.clear()
    }

    private suspend fun record(plan: RestructurePlanModel, moves: List<PlannedMoveModel>, run: Run): String? {
        if (moves.isEmpty()) {
            return null
        }
        val library = store.current() ?: return null
        val now = clock.now()
        val record = RestructureRecordModel(
            name = NAME_FORMAT.format(now),
            appliedAt = now,
            template = plan.template,
            moves = moves,
        )
        val replication = replicator.replicate(
            folders = library.folders,
            onPrimary = { primary -> plans.write(folder = primary, record = record) },
            followUp = FolderOperation.CopyPlan(name = record.name),
        )
        return when (replication) {
            is Replication.PrimaryFailed -> null
            is Replication.Done -> {
                run.pending += replication.pendingFolders
                record.name
            }
        }
    }

    /** Source folders the catalog has no documents in any more and whose primary copy holds nothing visible. */
    private suspend fun emptyFolders(sources: Set<RelativePathModel>): List<RelativePathModel> {
        val library = store.current() ?: return emptyList()
        return sources.filterNot { it.isRoot }.filter { folder ->
            val catalogued = library.catalog.documents.values.any { it.path.isInside(folder = folder) }
            val listed = storage.listNames(folder = library.folders.primary, path = folder)
            !catalogued && listed is StorageResult.Success && listed.value.all { it.startsWith(prefix = ".") }
        }.sortedBy { it.value }
    }

    private class Run {
        val batch: MutableList<PlannedMoveModel> = mutableListOf()
        val done: MutableList<PlannedMoveModel> = mutableListOf()
        val failed: MutableList<PlannedMoveModel> = mutableListOf()
        val notCatalogued: MutableList<PlannedMoveModel> = mutableListOf()
        val pending: MutableSet<LibraryFolderIdModel> = mutableSetOf()
    }

    private companion object {
        const val BATCH_SIZE = 50
        val NAME_FORMAT: DateTimeFormatter = DateTimeFormatter.ofPattern("yyyyMMdd-HHmmss", Locale.ROOT)
            .withZone(ZoneOffset.UTC)
    }
}
