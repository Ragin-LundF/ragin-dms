package dms.ui

import dms.application.catalog.EditCatalog
import dms.application.classification.RejectSuggestion
import dms.application.common.ActivityRecorder
import dms.application.common.SiblingNames
import dms.application.document.AddDocuments
import dms.application.document.AdoptNewFile
import dms.application.document.MoveDocument
import dms.application.document.RestoreDocument
import dms.application.document.TrashDocument
import dms.application.folder.CreateFolder
import dms.application.folder.MoveFolder
import dms.application.library.BuildDashboard
import dms.application.library.CatalogStore
import dms.application.library.ConfigureLibrary
import dms.application.library.OpenLibrary
import dms.application.library.ScanLibrary
import dms.application.port.system.IdGenerator
import dms.application.resolution.DismissTask
import dms.application.resolution.KeepBothVersions
import dms.application.resolution.KeepVersion
import dms.application.resolution.ResolveMove
import dms.application.sync.MergeAllFolders
import dms.application.sync.Replicator
import dms.application.sync.RetryFailedOperations
import dms.application.sync.SyncFromPrimary
import dms.storage.StorageAdapters
import dms.storage.platform.PlatformTrash
import dms.storage.platform.SystemTrash
import dms.ui.service.AppServices
import dms.ui.service.DocumentServices
import dms.ui.service.LibraryServices
import dms.ui.service.SyncServices
import dms.ui.system.SystemClock
import java.nio.file.Path
import java.util.UUID

/**
 * The composition root: the only place that creates adapters and use cases. Local state lives in
 * [appDataDirectory]; the library folders are whatever the settings there name.
 */
class AppContainer(appDataDirectory: Path, trash: SystemTrash = PlatformTrash()) {

    private val adapters = StorageAdapters(appDataDirectory = appDataDirectory, trash = trash)
    private val clock = SystemClock()
    private val ids = IdGenerator { UUID.randomUUID().toString() }
    private val replicator = Replicator(
        storage = adapters.documents,
        catalogs = adapters.catalogs,
        texts = adapters.texts,
        journal = adapters.journal,
        clock = clock,
        ids = ids,
    )
    private val store = CatalogStore(settings = adapters.settings, replicator = replicator)
    private val activity = ActivityRecorder(activity = adapters.activity, clock = clock)
    private val names = SiblingNames(storage = adapters.documents)
    private val openLibrary = OpenLibrary(
        settings = adapters.settings,
        catalogs = adapters.catalogs,
        replicator = replicator,
        store = store,
        tasks = adapters.tasks,
        ids = ids,
    )

    val services: AppServices = AppServices(
        library = libraryServices(),
        documents = documentServices(),
        sync = syncServices(),
    )

    private fun libraryServices(): LibraryServices {
        return LibraryServices(
            catalog = store.catalog,
            settings = adapters.settings,
            clock = clock,
            configureLibrary = ConfigureLibrary(settings = adapters.settings, ids = ids),
            openLibrary = openLibrary,
            scanLibrary = ScanLibrary(store = store, scanner = adapters.scanner),
            buildDashboard = BuildDashboard(
                store = store,
                tasks = adapters.tasks,
                journal = adapters.journal,
                activity = adapters.activity,
            ),
        )
    }

    private fun documentServices(): DocumentServices {
        val storage = adapters.documents
        return DocumentServices(
            addDocuments = AddDocuments(
                store = store,
                storage = storage,
                replicator = replicator,
                names = names,
                activity = activity,
                clock = clock,
                ids = ids,
            ),
            adoptNewFile = AdoptNewFile(
                store = store,
                storage = storage,
                replicator = replicator,
                activity = activity,
                clock = clock,
                ids = ids,
            ),
            moveDocument = MoveDocument(
                store = store,
                storage = storage,
                replicator = replicator,
                names = names,
                activity = activity,
            ),
            trashDocument = TrashDocument(
                store = store,
                replicator = replicator,
                tasks = adapters.tasks,
                activity = activity,
            ),
            restoreDocument = RestoreDocument(
                store = store,
                storage = storage,
                replicator = replicator,
                activity = activity,
            ),
            createFolder = CreateFolder(
                store = store,
                storage = storage,
                replicator = replicator,
                names = names,
                activity = activity,
            ),
            moveFolder = MoveFolder(
                store = store,
                storage = storage,
                replicator = replicator,
                names = names,
                activity = activity,
            ),
            editCatalog = EditCatalog(store = store, activity = activity, ids = ids),
        )
    }

    private fun syncServices(): SyncServices {
        val storage = adapters.documents
        return SyncServices(
            syncFromPrimary = SyncFromPrimary(
                store = store,
                scanner = adapters.scanner,
                replicator = replicator,
                activity = activity,
            ),
            mergeAllFolders = MergeAllFolders(
                store = store,
                scanner = adapters.scanner,
                storage = storage,
                replicator = replicator,
                openLibrary = openLibrary,
                activity = activity,
            ),
            retryFailedOperations = RetryFailedOperations(
                store = store,
                journal = adapters.journal,
                replicator = replicator,
            ),
            keepVersion = KeepVersion(store = store, storage = storage, replicator = replicator, activity = activity),
            keepBothVersions = KeepBothVersions(
                store = store,
                storage = storage,
                replicator = replicator,
                names = names,
                activity = activity,
            ),
            resolveMove = ResolveMove(store = store, storage = storage, replicator = replicator, activity = activity),
            dismissTask = DismissTask(tasks = adapters.tasks),
            rejectSuggestion = RejectSuggestion(tasks = adapters.tasks),
        )
    }
}
