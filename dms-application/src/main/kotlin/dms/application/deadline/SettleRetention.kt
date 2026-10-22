package dms.application.deadline

import dms.application.common.ActivityRecorder
import dms.application.common.OperationOutcome
import dms.application.library.CatalogStore
import dms.application.port.state.model.ActivityKind
import dms.application.port.system.Clock
import dms.domain.catalog.CatalogUpdate
import dms.domain.document.model.DocumentIdModel
import dms.domain.retention.model.RetentionModel

/**
 * Marks the ended retention of documents as done: they stay ordinary documents and are never flagged
 * again, also not through their category. One catalog change for all of them.
 */
class SettleRetention(
    private val store: CatalogStore,
    private val activity: ActivityRecorder,
    private val clock: Clock,
) {

    suspend operator fun invoke(documentIds: List<DocumentIdModel>): OperationOutcome {
        val now = clock.now()
        val outcome = store.commit { catalog ->
            val start: CatalogUpdate = CatalogUpdate.Applied(catalog = catalog)
            documentIds.fold(initial = start) { update, documentId ->
                update.andThen { current ->
                    val kept = current.documents[documentId]?.retention ?: RetentionModel()
                    current.setRetention(documentId = documentId, retention = kept.copy(settledAt = now))
                }
            }
        }
        if (outcome is OperationOutcome.Done) {
            activity.record(
                kind = ActivityKind.CATALOG_EDITED,
                subject = "Retention done for ${documentIds.size} documents",
            )
        }
        return outcome
    }
}
