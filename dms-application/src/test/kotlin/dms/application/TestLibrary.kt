package dms.application

import dms.application.ask.AskLibrary
import dms.application.ask.DocumentTexts
import dms.application.ask.IndexLibrary
import dms.application.ask.RelatedDocuments
import dms.application.ask.VectorSearch
import dms.application.assistant.AskAboutDocument
import dms.application.assistant.DocumentReading
import dms.application.assistant.SuggestTags
import dms.application.assistant.SummarizeDocument
import dms.application.assistant.TagAllUntagged
import dms.application.catalog.ChangeSettings
import dms.application.catalog.EditCatalog
import dms.application.classification.ApproveSuggestion
import dms.application.classification.ClassifyAllNew
import dms.application.classification.RejectSuggestion
import dms.application.classification.SuggestClassification
import dms.application.common.ActivityRecorder
import dms.application.common.DocumentDates
import dms.application.common.DocumentFiling
import dms.application.common.OperationOutcome
import dms.application.common.SiblingNames
import dms.application.deadline.SettleRetention
import dms.application.document.AddDocuments
import dms.application.document.AdoptNewFile
import dms.application.document.EditDocumentDetails
import dms.application.document.MoveDocument
import dms.application.document.RestoreDocument
import dms.application.document.TrashDocument
import dms.application.document.TrashDuplicateFile
import dms.application.document.TrashNewFile
import dms.application.document.model.AddDocumentsOutcome
import dms.application.document.model.AddedFileModel
import dms.application.export.ExportDocuments
import dms.application.fake.ai.FakeAssistant
import dms.application.fake.ai.FakeClassifier
import dms.application.fake.ai.FakeContentReader
import dms.application.fake.ai.FakeEmbeddings
import dms.application.fake.ai.FakeLibraryAssistant
import dms.application.fake.export.FakeExportWriter
import dms.application.fake.index.InMemoryVectorIndex
import dms.application.fake.pdf.FakeOcr
import dms.application.fake.pdf.FakePdfEditor
import dms.application.fake.state.InMemoryActivityLog
import dms.application.fake.state.InMemorySettings
import dms.application.fake.state.InMemoryTaskStore
import dms.application.fake.storage.FakeFolders
import dms.application.fake.storage.InMemoryJournal
import dms.application.fake.storage.InMemoryPlanStore
import dms.application.fake.system.FixedClock
import dms.application.fake.system.SequentialIds
import dms.application.folder.CreateFolder
import dms.application.folder.MoveFolder
import dms.application.folder.ReadFolderEntries
import dms.application.folder.ReadFolders
import dms.application.folder.TrashFolder
import dms.application.inbox.CollectInbox
import dms.application.inbox.ConfigureInbox
import dms.application.inbox.FileInboxDocument
import dms.application.library.AddScannedFiles
import dms.application.library.BuildDashboard
import dms.application.library.CatalogStore
import dms.application.library.DetectDocumentDates
import dms.application.library.OpenLibrary
import dms.application.library.ScanLibrary
import dms.application.library.model.DashboardModel
import dms.application.library.model.ScanProgressModel
import dms.application.library.model.ScanResultModel
import dms.application.library.model.outcome.DashboardOutcome
import dms.application.library.model.outcome.OpenLibraryOutcome
import dms.application.library.model.outcome.ScanOutcome
import dms.application.pdf.EditPages
import dms.application.pdf.EmbedPdfMetadata
import dms.application.pdf.MakeAllSearchable
import dms.application.pdf.MakeSearchable
import dms.application.pdf.MergeDocuments
import dms.application.pdf.PdfVersions
import dms.application.pdf.ScansWithoutText
import dms.application.pdf.SplitDocument
import dms.application.port.storage.model.ExternalFileModel
import dms.application.resolution.DismissTask
import dms.application.resolution.KeepBothVersions
import dms.application.resolution.KeepVersion
import dms.application.resolution.MergeDuplicates
import dms.application.resolution.ResolveMove
import dms.application.restructure.ApplyRestructurePlan
import dms.application.restructure.ListRestructureHistory
import dms.application.restructure.PlanRestructure
import dms.application.restructure.RemoveEmptyFolders
import dms.application.restructure.ReverseRestructure
import dms.application.rule.AcceptRuleProposal
import dms.application.search.ContentMatcher
import dms.application.search.FindDocuments
import dms.application.search.SearchContents
import dms.application.sync.MergeAllFolders
import dms.application.sync.Replicator
import dms.application.sync.RetryFailedOperations
import dms.application.sync.SyncFromPrimary
import dms.application.sync.model.RetryOutcome
import dms.application.sync.model.RetryReportModel
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.FolderRole
import dms.domain.library.model.FolderLocationModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.LibraryFoldersModel
import dms.domain.library.model.RelativePathModel
import java.time.Instant
import kotlin.test.assertIs
import kotlin.test.assertNotNull

/**
 * A library with a primary and two sync folders, all in memory, wired with every use case. Only
 * synthetic names and contents; never real documents.
 */
class TestLibrary {

    val primary: LibraryFolderModel = folder(id = "primary", role = FolderRole.PRIMARY)
    val syncA: LibraryFolderModel = folder(id = "sync-a", role = FolderRole.SYNC)
    val syncB: LibraryFolderModel = folder(id = "sync-b", role = FolderRole.SYNC)
    val allFolders: List<LibraryFolderModel> = listOf(primary, syncA, syncB)

    val folders: FakeFolders = FakeFolders()
    val settings: InMemorySettings = InMemorySettings(folders = LibraryFoldersModel(folders = allFolders))
    val journal: InMemoryJournal = InMemoryJournal()
    val tasks: InMemoryTaskStore = InMemoryTaskStore()
    val activityLog: InMemoryActivityLog = InMemoryActivityLog()
    val clock: FixedClock = FixedClock()
    val ids: SequentialIds = SequentialIds()
    val classifier: FakeClassifier = FakeClassifier()
    val reader: FakeContentReader = FakeContentReader()
    val assistant: FakeAssistant = FakeAssistant()
    val plans: InMemoryPlanStore = InMemoryPlanStore(unreachable = folders.unreachable)
    val index: InMemoryVectorIndex = InMemoryVectorIndex(unreachable = folders.unreachable)
    val embeddings: FakeEmbeddings = FakeEmbeddings()
    val libraryAssistant: FakeLibraryAssistant = FakeLibraryAssistant()
    val exports: FakeExportWriter = FakeExportWriter()
    val pdf: FakePdfEditor = FakePdfEditor(folders = folders)
    val ocr: FakeOcr = FakeOcr(folders = folders)

    val replicator: Replicator = Replicator(
        storage = folders,
        catalogs = folders,
        texts = folders,
        plans = plans,
        index = index,
        journal = journal,
        clock = clock,
        ids = ids,
    )
    val store: CatalogStore = CatalogStore(settings = settings, catalogs = folders, replicator = replicator)
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
        clock = clock,
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
    val trashNewFile: TrashNewFile = TrashNewFile(store = store, storage = folders, activity = activity)
    val trashDuplicateFile: TrashDuplicateFile = TrashDuplicateFile(
        store = store,
        storage = folders,
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
    val readFolderEntries: ReadFolderEntries = ReadFolderEntries(store = store, storage = folders)
    val readFolders: ReadFolders = ReadFolders(store = store, storage = folders)
    val trashFolder: TrashFolder = TrashFolder(
        store = store,
        replicator = replicator,
        tasks = tasks,
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
    val mergeDuplicates: MergeDuplicates = MergeDuplicates(
        store = store,
        storage = folders,
        tasks = tasks,
        activity = activity,
    )

    val suggestClassification: SuggestClassification = SuggestClassification(
        store = store,
        reader = reader,
        classifier = classifier,
        tasks = tasks,
        texts = folders,
        replicator = replicator,
        clock = clock,
    )
    val filing: DocumentFiling = DocumentFiling(store = store, names = names, moveDocument = moveDocument)
    val approveSuggestion: ApproveSuggestion = ApproveSuggestion(
        store = store,
        tasks = tasks,
        filing = filing,
        activity = activity,
        clock = clock,
        ids = ids,
    )
    val rejectSuggestion: RejectSuggestion = RejectSuggestion(tasks = tasks)
    val classifyAllNew: ClassifyAllNew = ClassifyAllNew(store = store, tasks = tasks, suggest = suggestClassification)

    private val reading: DocumentReading = DocumentReading(store = store, reader = reader)
    val summarizeDocument: SummarizeDocument = SummarizeDocument(reading = reading, assistant = assistant)
    val suggestTags: SuggestTags = SuggestTags(reading = reading, assistant = assistant, tasks = tasks)
    val tagAllUntagged: TagAllUntagged = TagAllUntagged(store = store, tasks = tasks, suggestTags = suggestTags)
    val askAboutDocument: AskAboutDocument = AskAboutDocument(store = store, reading = reading, assistant = assistant)
    private val matcher: ContentMatcher = ContentMatcher(reader = reader, texts = folders, clock = clock)
    val searchContents: SearchContents = SearchContents(store = store, matcher = matcher)
    val findDocuments: FindDocuments = FindDocuments(store = store, matcher = matcher)

    val editDocumentDetails: EditDocumentDetails = EditDocumentDetails(store = store, activity = activity)
    val settleRetention: SettleRetention = SettleRetention(store = store, activity = activity, clock = clock)
    val changeSettings: ChangeSettings = ChangeSettings(store = store, activity = activity)
    val acceptRuleProposal: AcceptRuleProposal = AcceptRuleProposal(changeSettings = changeSettings, ids = ids)
    val exportDocuments: ExportDocuments = ExportDocuments(
        store = store,
        search = matcher,
        writer = exports,
        activity = activity,
        clock = clock,
    )
    private val pdfVersions: PdfVersions = PdfVersions(
        store = store,
        storage = folders,
        replicator = replicator,
        reader = reader,
        texts = folders,
        ids = ids,
    )
    val editPages: EditPages = EditPages(store = store, pdf = pdf, versions = pdfVersions, activity = activity)
    val splitDocument: SplitDocument = SplitDocument(
        store = store,
        pdf = pdf,
        replicator = replicator,
        names = names,
        trashDocument = trashDocument,
        activity = activity,
        clock = clock,
        ids = ids,
    )
    val mergeDocuments: MergeDocuments = MergeDocuments(
        store = store,
        pdf = pdf,
        replicator = replicator,
        names = names,
        trashDocument = trashDocument,
        activity = activity,
        clock = clock,
        ids = ids,
    )
    val makeSearchable: MakeSearchable = MakeSearchable(
        store = store,
        pdf = pdf,
        ocr = ocr,
        texts = folders,
        versions = pdfVersions,
        activity = activity,
    )
    val makeAllSearchable: MakeAllSearchable = MakeAllSearchable(store = store, makeSearchable = makeSearchable)
    val scansWithoutText: ScansWithoutText = ScansWithoutText(store = store, pdf = pdf)
    val embedPdfMetadata: EmbedPdfMetadata = EmbedPdfMetadata(store = store, pdf = pdf, versions = pdfVersions)
    private val documentTexts: DocumentTexts = DocumentTexts(reader = reader, texts = folders)
    private val vectorSearch: VectorSearch = VectorSearch(index = index)
    val indexLibrary: IndexLibrary = IndexLibrary(
        store = store,
        texts = documentTexts,
        embeddings = embeddings,
        index = index,
        replicator = replicator,
    )
    val askLibrary: AskLibrary = AskLibrary(
        store = store,
        search = matcher,
        texts = documentTexts,
        embeddings = embeddings,
        vectors = vectorSearch,
        assistant = libraryAssistant,
    )
    val relatedDocuments: RelatedDocuments = RelatedDocuments(store = store, vectors = vectorSearch)
    val configureInbox: ConfigureInbox = ConfigureInbox(
        store = store,
        storage = folders,
        replicator = replicator,
        activity = activity,
    )
    val collectInbox: CollectInbox = CollectInbox(store = store, adoptNewFile = adoptNewFile)
    val fileInboxDocument: FileInboxDocument = FileInboxDocument(store = store, filing = filing, activity = activity)
    val planRestructure: PlanRestructure = PlanRestructure(store = store, search = matcher)
    val applyRestructurePlan: ApplyRestructurePlan = ApplyRestructurePlan(
        store = store,
        storage = folders,
        replicator = replicator,
        plans = plans,
        activity = activity,
        clock = clock,
    )
    val reverseRestructure: ReverseRestructure = ReverseRestructure(
        store = store,
        plans = plans,
        apply = applyRestructurePlan,
    )
    val listRestructureHistory: ListRestructureHistory = ListRestructureHistory(store = store, plans = plans)
    val removeEmptyFolders: RemoveEmptyFolders = RemoveEmptyFolders(
        store = store,
        storage = folders,
        replicator = replicator,
        activity = activity,
    )

    /** The open catalog; fails the test while no library is open. */
    val catalog: CatalogModel
        get() {
            return assertNotNull(actual = store.catalog.value)
        }

    /** Opens the library and returns the opened catalog. */
    suspend fun open(): CatalogModel {
        return assertIs<OpenLibraryOutcome.Opened>(value = openLibrary()).catalog
    }

    /** Adds a file with [content] at [path] to every folder through the tool and returns its document. */
    suspend fun add(path: String, content: String): DocumentModel {
        val target = RelativePathModel.of(raw = path)
        val source = folders.external(location = "/outside/${target.fileName}", content = content)
        val added = added(files = listOf(source), targetFolder = target.parent ?: RelativePathModel.ROOT).single()
        assertIs<OperationOutcome.Done>(value = added.outcome)
        return assertNotNull(actual = catalog.findByPath(path = target))
    }

    /** Scans like [scanLibrary]; `null` while no library is open. */
    suspend fun scanned(addNewFiles: Boolean = false, onProgress: (ScanProgressModel) -> Unit = {}): ScanResultModel? {
        val outcome = scanLibrary(addNewFiles = addNewFiles, onProgress = onProgress)
        return (outcome as? ScanOutcome.Scanned)?.result
    }

    /** The dashboard [buildDashboard] builds; `null` while no library is open. */
    suspend fun dashboard(scan: ScanResultModel?): DashboardModel? {
        return (buildDashboard(scan = scan) as? DashboardOutcome.Built)?.dashboard
    }

    /** The report of [retryFailedOperations]; `null` while no library is open. */
    suspend fun retried(): RetryReportModel? {
        return (retryFailedOperations() as? RetryOutcome.Retried)?.report
    }

    /** The files [addDocuments] added; fails the test while no library is open. */
    suspend fun added(files: List<ExternalFileModel>, targetFolder: RelativePathModel): List<AddedFileModel> {
        val outcome = addDocuments(files = files, targetFolder = targetFolder)
        return assertIs<AddDocumentsOutcome.Added>(value = outcome).files
    }

    private fun folder(id: String, role: FolderRole): LibraryFolderModel {
        return LibraryFolderModel(
            id = LibraryFolderIdModel(value = id),
            displayName = id,
            location = FolderLocationModel(value = "/library/$id"),
            role = role,
        )
    }

    companion object {

        /** A document for catalogs built by hand; [content] decides its hash and size. */
        fun document(id: String, path: String, content: String): DocumentModel {
            val facts = FakeFolders.factsOf(content = content)
            return DocumentModel(
                id = DocumentIdModel(value = id),
                path = RelativePathModel.of(raw = path),
                contentHash = FakeFolders.hashOf(content = content),
                size = facts.size,
                addedAt = Instant.parse("2024-01-01T10:00:00Z"),
            )
        }
    }
}
