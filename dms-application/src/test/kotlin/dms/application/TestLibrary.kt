package dms.application

import dms.application.catalog.EditCatalog
import dms.application.classification.ApproveSuggestion
import dms.application.classification.ClassifyAllNew
import dms.application.classification.RejectSuggestion
import dms.application.classification.SuggestClassification
import dms.application.common.ActivityRecorder
import dms.application.common.DocumentDates
import dms.application.common.OperationOutcome
import dms.application.common.SiblingNames
import dms.application.document.AddDocuments
import dms.application.document.AdoptNewFile
import dms.application.document.MoveDocument
import dms.application.document.RestoreDocument
import dms.application.document.TrashDocument
import dms.application.fake.ai.FakeClassifier
import dms.application.fake.ai.FakeContentReader
import dms.application.fake.state.InMemoryActivityLog
import dms.application.fake.state.InMemorySettings
import dms.application.fake.state.InMemoryTaskStore
import dms.application.fake.storage.FakeFolders
import dms.application.fake.storage.InMemoryJournal
import dms.application.fake.system.FixedClock
import dms.application.fake.system.SequentialIds
import dms.application.folder.CreateFolder
import dms.application.folder.MoveFolder
import dms.application.library.AddScannedFiles
import dms.application.library.BuildDashboard
import dms.application.library.CatalogStore
import dms.application.library.DetectDocumentDates
import dms.application.library.OpenLibrary
import dms.application.library.ScanLibrary
import dms.application.library.model.OpenLibraryOutcome
import dms.application.resolution.DismissTask
import dms.application.resolution.KeepBothVersions
import dms.application.resolution.KeepVersion
import dms.application.resolution.ResolveMove
import dms.application.sync.MergeAllFolders
import dms.application.sync.Replicator
import dms.application.sync.RetryFailedOperations
import dms.application.sync.SyncFromPrimary
import dms.domain.catalog.Catalog
import dms.domain.document.Document
import dms.domain.document.DocumentId
import dms.domain.library.FolderLocation
import dms.domain.library.FolderRole
import dms.domain.library.LibraryFolder
import dms.domain.library.LibraryFolderId
import dms.domain.library.LibraryFolders
import dms.domain.library.RelativePath
import java.time.Instant
import kotlin.test.assertIs
import kotlin.test.assertNotNull

/**
 * A library with a primary and two sync folders, all in memory, wired with every use case. Only
 * synthetic names and contents; never real documents.
 */
class TestLibrary {

    val primary: LibraryFolder = folder(id = "primary", role = FolderRole.PRIMARY)
    val syncA: LibraryFolder = folder(id = "sync-a", role = FolderRole.SYNC)
    val syncB: LibraryFolder = folder(id = "sync-b", role = FolderRole.SYNC)
    val allFolders: List<LibraryFolder> = listOf(primary, syncA, syncB)

    val folders: FakeFolders = FakeFolders()
    val settings: InMemorySettings = InMemorySettings(folders = LibraryFolders(folders = allFolders))
    val journal: InMemoryJournal = InMemoryJournal()
    val tasks: InMemoryTaskStore = InMemoryTaskStore()
    val activityLog: InMemoryActivityLog = InMemoryActivityLog()
    val clock: FixedClock = FixedClock()
    val ids: SequentialIds = SequentialIds()
    val classifier: FakeClassifier = FakeClassifier()
    val reader: FakeContentReader = FakeContentReader()

    val replicator: Replicator = Replicator(
        storage = folders,
        catalogs = folders,
        texts = folders,
        journal = journal,
        clock = clock,
        ids = ids,
    )
    val store: CatalogStore = CatalogStore(settings = settings, replicator = replicator)
    private val activity = ActivityRecorder(activity = activityLog, clock = clock)
    private val names = SiblingNames(storage = folders)
    private val dates = DocumentDates(storage = folders)

    val openLibrary: OpenLibrary = OpenLibrary(
        settings = settings,
        catalogs = folders,
        replicator = replicator,
        store = store,
        tasks = tasks,
        ids = ids,
    )
    val addScannedFiles: AddScannedFiles = AddScannedFiles(
        store = store,
        dates = dates,
        activity = activity,
        clock = clock,
        ids = ids,
    )
    val detectDocumentDates: DetectDocumentDates = DetectDocumentDates(
        store = store,
        dates = dates,
        activity = activity,
    )
    val scanLibrary: ScanLibrary = ScanLibrary(store = store, scanner = folders, addScannedFiles = addScannedFiles)
    val buildDashboard: BuildDashboard = BuildDashboard(
        store = store,
        tasks = tasks,
        journal = journal,
        activity = activityLog,
    )

    val addDocuments: AddDocuments = AddDocuments(
        store = store,
        storage = folders,
        replicator = replicator,
        names = names,
        activity = activity,
        clock = clock,
        ids = ids,
    )
    val adoptNewFile: AdoptNewFile = AdoptNewFile(
        store = store,
        storage = folders,
        dates = dates,
        replicator = replicator,
        activity = activity,
        clock = clock,
        ids = ids,
    )
    val moveDocument: MoveDocument = MoveDocument(
        store = store,
        storage = folders,
        replicator = replicator,
        names = names,
        activity = activity,
    )
    val trashDocument: TrashDocument = TrashDocument(
        store = store,
        replicator = replicator,
        tasks = tasks,
        activity = activity,
    )
    val restoreDocument: RestoreDocument = RestoreDocument(
        store = store,
        storage = folders,
        replicator = replicator,
        activity = activity,
    )

    val createFolder: CreateFolder = CreateFolder(
        store = store,
        storage = folders,
        replicator = replicator,
        names = names,
        activity = activity,
    )
    val moveFolder: MoveFolder = MoveFolder(
        store = store,
        storage = folders,
        replicator = replicator,
        names = names,
        activity = activity,
    )
    val editCatalog: EditCatalog = EditCatalog(store = store, activity = activity, ids = ids)

    val syncFromPrimary: SyncFromPrimary = SyncFromPrimary(
        store = store,
        scanner = folders,
        replicator = replicator,
        activity = activity,
    )
    val mergeAllFolders: MergeAllFolders = MergeAllFolders(
        store = store,
        scanner = folders,
        storage = folders,
        replicator = replicator,
        openLibrary = openLibrary,
        activity = activity,
    )
    val retryFailedOperations: RetryFailedOperations = RetryFailedOperations(
        store = store,
        journal = journal,
        replicator = replicator,
    )

    val keepVersion: KeepVersion = KeepVersion(
        store = store,
        storage = folders,
        replicator = replicator,
        activity = activity,
    )
    val keepBothVersions: KeepBothVersions = KeepBothVersions(
        store = store,
        storage = folders,
        replicator = replicator,
        names = names,
        activity = activity,
    )
    val resolveMove: ResolveMove = ResolveMove(
        store = store,
        storage = folders,
        replicator = replicator,
        activity = activity,
    )
    val dismissTask: DismissTask = DismissTask(tasks = tasks)

    val suggestClassification: SuggestClassification = SuggestClassification(
        store = store,
        reader = reader,
        classifier = classifier,
        tasks = tasks,
        texts = folders,
        replicator = replicator,
        clock = clock,
    )
    val approveSuggestion: ApproveSuggestion = ApproveSuggestion(
        store = store,
        tasks = tasks,
        moveDocument = moveDocument,
        activity = activity,
        clock = clock,
        ids = ids,
    )
    val rejectSuggestion: RejectSuggestion = RejectSuggestion(tasks = tasks)
    val classifyAllNew: ClassifyAllNew = ClassifyAllNew(store = store, tasks = tasks, suggest = suggestClassification)

    /** The open catalog; fails the test while no library is open. */
    val catalog: Catalog
        get() {
            return assertNotNull(actual = store.catalog.value)
        }

    /** Opens the library and returns the opened catalog. */
    suspend fun open(): Catalog {
        return assertIs<OpenLibraryOutcome.Opened>(value = openLibrary()).catalog
    }

    /** Adds a file with [content] at [path] to every folder through the tool and returns its document. */
    suspend fun add(path: String, content: String): Document {
        val target = RelativePath.of(raw = path)
        val source = folders.external(location = "/outside/${target.fileName}", content = content)
        val added = addDocuments(files = listOf(source), targetFolder = target.parent ?: RelativePath.ROOT).single()
        assertIs<OperationOutcome.Done>(value = added.outcome)
        return assertNotNull(actual = catalog.findByPath(path = target))
    }

    private fun folder(id: String, role: FolderRole): LibraryFolder {
        return LibraryFolder(
            id = LibraryFolderId(value = id),
            displayName = id,
            location = FolderLocation(value = "/library/$id"),
            role = role,
        )
    }

    companion object {

        /** A document for catalogs built by hand; [content] decides its hash and size. */
        fun document(id: String, path: String, content: String): Document {
            val facts = FakeFolders.factsOf(content = content)
            return Document(
                id = DocumentId(value = id),
                path = RelativePath.of(raw = path),
                contentHash = FakeFolders.hashOf(content = content),
                size = facts.size,
                addedAt = Instant.parse("2024-01-01T10:00:00Z"),
            )
        }
    }
}
