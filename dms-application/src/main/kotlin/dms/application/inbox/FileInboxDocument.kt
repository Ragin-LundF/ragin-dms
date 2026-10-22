package dms.application.inbox

import dms.application.common.ActivityRecorder
import dms.application.common.DocumentFiling
import dms.application.common.OperationOutcome
import dms.application.library.CatalogStore
import dms.application.port.state.model.ActivityKind
import dms.domain.catalog.CatalogRejection
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.RelativePathModel

/**
 * Files an inbox document by hand, without any AI: one move into [targetFolder] in every library
 * folder, optionally with a new name. A missing target folder is created; a taken name gets a
 * ` (2)` suffix. Once the document left the inbox, it is an ordinary document.
 */
class FileInboxDocument(
    private val store: CatalogStore,
    private val filing: DocumentFiling,
    private val activity: ActivityRecorder,
) {

    suspend operator fun invoke(
        documentId: DocumentIdModel,
        targetFolder: RelativePathModel,
        newName: String?,
    ): OperationOutcome {
        val library = store.current() ?: return OperationOutcome.LibraryNotOpen
        val inbox = library.catalog.settings.layout.inbox
        if (inbox.enabled && (targetFolder == inbox.path || targetFolder.isInside(folder = inbox.path))) {
            return OperationOutcome.Rejected(reason = CatalogRejection.INVALID_FOLDER_MOVE)
        }
        val outcome = filing.file(documentId = documentId, folder = targetFolder, fileName = newName)
        if (outcome is OperationOutcome.Done) {
            val path = outcome.catalog.documents[documentId]?.path?.value ?: targetFolder.value
            activity.record(kind = ActivityKind.DOCUMENT_FILED, subject = path)
        }
        return outcome
    }
}
