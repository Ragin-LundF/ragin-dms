package dms.application.inbox

import dms.application.common.ActivityRecorder
import dms.application.common.OperationOutcome
import dms.application.library.CatalogStore
import dms.application.port.state.model.ActivityKind
import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.model.FolderOperation
import dms.application.sync.Replicator
import dms.application.sync.model.Replication
import dms.domain.catalog.CatalogUpdate
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.settings.model.InboxSettingsModel

/**
 * Saves the inbox settings to the catalog, so every device uses them. Their path was checked when
 * they were created (see [InboxSettingsModel.pathViolation]). Switching the inbox on creates
 * its folder in every library folder first; an existing folder is kept as it is.
 */
class ConfigureInbox(
    private val store: CatalogStore,
    private val storage: DocumentStorage,
    private val replicator: Replicator,
    private val activity: ActivityRecorder,
) {

    suspend operator fun invoke(inbox: InboxSettingsModel): OperationOutcome {
        val library = store.current() ?: return OperationOutcome.LibraryNotOpen
        var pending = emptySet<LibraryFolderIdModel>()
        if (inbox.enabled) {
            val created = replicator.replicate(
                folders = library.folders,
                onPrimary = { primary -> storage.createFolder(folder = primary, path = inbox.path) },
                followUp = FolderOperation.CreateFolder(path = inbox.path),
            )
            when (created) {
                is Replication.PrimaryFailed -> return OperationOutcome.StorageFailed(failure = created.failure)
                is Replication.Done -> pending = created.pendingFolders
            }
        }
        val outcome = store.commit { catalog ->
            val layout = catalog.settings.layout.copy(inbox = inbox)
            if (layout == catalog.settings.layout) {
                CatalogUpdate.Applied(catalog = catalog)
            } else {
                catalog.changeSettings(settings = catalog.settings.withLayout(layout = layout))
            }
        }.withPendingFolders(folders = pending)
        if (outcome is OperationOutcome.Done) {
            activity.record(kind = ActivityKind.CATALOG_EDITED, subject = "Inbox ${inbox.path}")
        }
        return outcome
    }
}
