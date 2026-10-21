package dms.application.library

import dms.application.common.ActivityRecorder
import dms.application.common.DocumentDates
import dms.application.common.OperationOutcome
import dms.application.port.state.model.ActivityKind
import dms.domain.catalog.CatalogRejection
import dms.domain.catalog.CatalogUpdate
import dms.domain.document.DocumentId
import java.time.LocalDate

/**
 * Gives every document without a date the one [DocumentDates] finds, for documents catalogued before
 * dates were detected. Dates the user or an approved suggestion set are never touched.
 */
class DetectDocumentDates(
    private val store: CatalogStore,
    private val dates: DocumentDates,
    private val activity: ActivityRecorder,
) {

    /** Returns how many documents got a date. */
    suspend operator fun invoke(): Int {
        val library = store.current() ?: return 0
        val found = mutableMapOf<DocumentId, LocalDate>()
        library.catalog.documents.values
            .filter { it.metadata.documentDate == null }
            .forEach { document ->
                dates.guess(folder = library.folders.primary, path = document.path)?.let { found[document.id] = it }
            }
        if (found.isEmpty()) {
            return 0
        }
        var dated = 0
        val outcome = store.commit { catalog ->
            var current = catalog
            dated = 0
            found.forEach { (id, date) ->
                val document = current.documents[id]
                if (document != null && document.metadata.documentDate == null) {
                    val update = current.updateMetadata(
                        documentId = id,
                        metadata = document.metadata.copy(documentDate = date),
                    )
                    if (update is CatalogUpdate.Applied) {
                        current = update.catalog
                        dated++
                    }
                }
            }
            if (dated == 0) {
                CatalogUpdate.Rejected(reason = CatalogRejection.DOCUMENT_NOT_FOUND)
            } else {
                CatalogUpdate.Applied(catalog = current)
            }
        }
        if (outcome !is OperationOutcome.Done) {
            return 0
        }
        activity.record(kind = ActivityKind.CATALOG_EDITED, subject = "Dates of $dated documents")
        return dated
    }
}
