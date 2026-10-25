package dms.application.inbox

import dms.application.common.OperationOutcome
import dms.application.document.AdoptNewFile
import dms.application.inbox.model.CollectInboxOutcome
import dms.application.library.CatalogStore
import dms.application.library.model.ScanResultModel
import dms.domain.change.DetectedChange

/**
 * Catalogs the new files a scan found in the inbox of any library folder. A file that arrived in a
 * sync folder is copied into the primary first, verified, and from there into the other folders
 * (see [AdoptNewFile]). Copies of a catalogued document are left to the dashboard.
 */
class CollectInbox(
    private val store: CatalogStore,
    private val adoptNewFile: AdoptNewFile,
) {

    suspend operator fun invoke(scan: ScanResultModel): CollectInboxOutcome {
        val library = store.current() ?: return CollectInboxOutcome.LibraryNotOpen
        val inbox = library.catalog.settings.layout.inbox
        if (!inbox.enabled) {
            return CollectInboxOutcome.Collected(changes = emptyList())
        }
        val known = library.catalog.documents.values.map { it.contentHash }.toSet()
        val collected = scan.changes.filterIsInstance<DetectedChange.NewFile>()
            .filter { inbox.contains(candidate = it.path) && it.contentHash != null && it.contentHash !in known }
            .distinctBy { it.contentHash }
            .filter { adoptNewFile(change = it) is OperationOutcome.Done }
        return CollectInboxOutcome.Collected(changes = collected)
    }
}
