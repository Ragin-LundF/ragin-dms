package dms.ui

import dms.ai.AiAdapters
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
import dms.application.classification.ListAiModels
import dms.application.classification.RejectSuggestion
import dms.application.classification.SuggestClassification
import dms.application.common.ActivityRecorder
import dms.application.common.DocumentDates
import dms.application.common.DocumentFiling
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
import dms.application.export.ExportDocuments
import dms.application.folder.CreateFolder
import dms.application.folder.MoveFolder
import dms.application.inbox.CollectInbox
import dms.application.inbox.ConfigureInbox
import dms.application.inbox.FileInboxDocument
import dms.application.library.AddScannedFiles
import dms.application.library.BuildDashboard
import dms.application.library.CatalogStore
import dms.application.library.ConfigureLibrary
import dms.application.library.DetectDocumentDates
import dms.application.library.OpenLibrary
import dms.application.library.ScanLibrary
import dms.application.pdf.EditPages
import dms.application.pdf.EmbedPdfMetadata
import dms.application.pdf.MakeAllSearchable
import dms.application.pdf.MakeSearchable
import dms.application.pdf.MergeDocuments
import dms.application.pdf.PdfVersions
import dms.application.pdf.ScansWithoutText
import dms.application.pdf.SplitDocument
import dms.application.port.system.IdGenerator
import dms.application.preview.DocumentThumbnails
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
import dms.pdf.PdfAdapters
import dms.pdf.io.PdfLocations
import dms.storage.StorageAdapters
import dms.storage.fs.PathResolver
import dms.storage.platform.PlatformTrash
import dms.storage.platform.SystemTrash
import dms.ui.service.AiServices
import dms.ui.service.AppServices
import dms.ui.service.AskServices
import dms.ui.service.DocumentServices
import dms.ui.service.LibraryServices
import dms.ui.service.OrganizeServices
import dms.ui.service.PdfServices
import dms.ui.service.SyncServices
import dms.ui.system.SystemClock
import io.ktor.client.engine.HttpClientEngine
import java.nio.file.Path
import java.util.UUID

/**
 * The composition root: the only place that creates adapters and use cases. Local state lives in
 * [appDataDirectory]; the library folders are whatever the settings there name. Tests pass an
 * [aiEngine] that answers in place of the AI server.
 */
class AppContainer(
    appDataDirectory: Path,
    trash: SystemTrash = PlatformTrash(),
    aiEngine: HttpClientEngine? = null,
) {

    private val adapters = StorageAdapters(appDataDirectory = appDataDirectory, trash = trash)
    private val clock = SystemClock()
    private val ids = IdGenerator { UUID.randomUUID().toString() }
    private val replicator = Replicator(
        storage = adapters.documents,
        catalogs = adapters.catalogs,
        texts = adapters.texts,
        plans = adapters.plans,
        index = adapters.index,
        journal = adapters.journal,
        clock = clock,
        ids = ids,
    )
    private val store = CatalogStore(
        settings = adapters.settings,
        catalogs = adapters.catalogs,
        replicator = replicator,
    )
    private val activity = ActivityRecorder(activity = adapters.activity, clock = clock)
    private val names = SiblingNames(storage = adapters.documents)
    private val dates = DocumentDates(storage = adapters.documents)
    private val ai = AiAdapters(
        settings = { adapters.settings.loadPreferences().ai },
        locate = { folder, path ->
            PathResolver.root(folder = folder)?.let { root -> PathResolver.existing(root = root, path = path) }
        },
        engine = aiEngine,
    )
    private val pdf = PdfAdapters(
        locations = PdfLocations(
            existing = { folder, path ->
                PathResolver.root(folder = folder)?.let { PathResolver.existing(root = it, path = path) }
            },
            target = { folder, path ->
                PathResolver.root(folder = folder)?.let { PathResolver.target(root = it, path = path) }
            },
        ),
    )
    private val trashDocument = TrashDocument(
        store = store,
        replicator = replicator,
        tasks = adapters.tasks,
        activity = activity,
    )
    private val moveDocument = MoveDocument(
        store = store,
        storage = adapters.documents,
        replicator = replicator,
        names = names,
        activity = activity,
    )
    private val changeSettings = ChangeSettings(store = store, activity = activity)
    private val filing = DocumentFiling(store = store, names = names, moveDocument = moveDocument)
    private val matcher = ContentMatcher(reader = ai.reader, texts = adapters.texts, clock = clock)
    private val adoptNewFile = AdoptNewFile(
        store = store,
        storage = adapters.documents,
        dates = dates,
        replicator = replicator,
        activity = activity,
        clock = clock,
        ids = ids,
    )
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
        ai = aiServices(),
        organize = organizeServices(),
        pdf = pdfServices(),
        ask = askServices(),
    )

    private fun libraryServices(): LibraryServices {
        return LibraryServices(
            catalog = store.catalog,
            settings = adapters.settings,
            clock = clock,
            configureLibrary = ConfigureLibrary(settings = adapters.settings, ids = ids),
            openLibrary = openLibrary,
            scanLibrary = ScanLibrary(
                store = store,
                scanner = adapters.scanner,
                addScannedFiles = AddScannedFiles(
                    store = store,
                    dates = dates,
                    activity = activity,
                    clock = clock,
                    ids = ids,
                ),
            ),
            detectDocumentDates = DetectDocumentDates(store = store, dates = dates, activity = activity),
            buildDashboard = BuildDashboard(
                store = store,
                tasks = adapters.tasks,
                journal = adapters.journal,
                activity = adapters.activity,
                clock = clock,
            ),
            searchContents = SearchContents(store = store, matcher = matcher),
            findDocuments = FindDocuments(store = store, matcher = matcher),
            thumbnails = DocumentThumbnails(store = store, previewer = ai.previewer),
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
            adoptNewFile = adoptNewFile,
            moveDocument = moveDocument,
            trashDocument = trashDocument,
            trashNewFile = TrashNewFile(store = store, storage = storage, activity = activity),
            trashDuplicateFile = TrashDuplicateFile(store = store, storage = storage, activity = activity),
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
            changeSettings = changeSettings,
            editDocumentDetails = EditDocumentDetails(store = store, activity = activity),
            settleRetention = SettleRetention(store = store, activity = activity, clock = clock),
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
            mergeDuplicates = MergeDuplicates(
                store = store,
                storage = storage,
                tasks = adapters.tasks,
                activity = activity,
            ),
        )
    }

    private fun aiServices(): AiServices {
        val suggest = SuggestClassification(
            store = store,
            reader = ai.reader,
            classifier = ai.client,
            tasks = adapters.tasks,
            texts = adapters.texts,
            replicator = replicator,
            clock = clock,
        )
        val reading = DocumentReading(store = store, reader = ai.reader)
        val suggestTags = SuggestTags(reading = reading, assistant = ai.assistant, tasks = adapters.tasks)
        return AiServices(
            suggestClassification = suggest,
            classifyAllNew = ClassifyAllNew(store = store, tasks = adapters.tasks, suggest = suggest),
            approveSuggestion = ApproveSuggestion(
                store = store,
                tasks = adapters.tasks,
                filing = filing,
                activity = activity,
                clock = clock,
                ids = ids,
            ),
            rejectSuggestion = RejectSuggestion(tasks = adapters.tasks),
            listAiModels = ListAiModels(server = ai.client),
            summarizeDocument = SummarizeDocument(reading = reading, assistant = ai.assistant),
            suggestTags = suggestTags,
            tagAllUntagged = TagAllUntagged(store = store, tasks = adapters.tasks, suggestTags = suggestTags),
            askAboutDocument = AskAboutDocument(store = store, reading = reading, assistant = ai.assistant),
        )
    }

    private fun organizeServices(): OrganizeServices {
        val apply = ApplyRestructurePlan(
            store = store,
            storage = adapters.documents,
            replicator = replicator,
            plans = adapters.plans,
            activity = activity,
            clock = clock,
        )
        return OrganizeServices(
            configureInbox = ConfigureInbox(
                store = store,
                storage = adapters.documents,
                replicator = replicator,
                activity = activity,
            ),
            collectInbox = CollectInbox(store = store, adoptNewFile = adoptNewFile),
            fileInboxDocument = FileInboxDocument(store = store, filing = filing, activity = activity),
            folderWatcher = adapters.watcher,
            planRestructure = PlanRestructure(store = store, search = matcher),
            applyRestructurePlan = apply,
            reverseRestructure = ReverseRestructure(store = store, plans = adapters.plans, apply = apply),
            listRestructureHistory = ListRestructureHistory(store = store, plans = adapters.plans),
            removeEmptyFolders = RemoveEmptyFolders(
                store = store,
                storage = adapters.documents,
                replicator = replicator,
                activity = activity,
            ),
            acceptRuleProposal = AcceptRuleProposal(changeSettings = changeSettings, ids = ids),
            exportDocuments = ExportDocuments(
                store = store,
                search = matcher,
                writer = adapters.exports,
                activity = activity,
                clock = clock,
            ),
            exportWriter = adapters.exports,
        )
    }

    private fun pdfServices(): PdfServices {
        val versions = PdfVersions(
            store = store,
            storage = adapters.documents,
            replicator = replicator,
            reader = ai.reader,
            texts = adapters.texts,
            ids = ids,
        )
        val makeSearchable = MakeSearchable(
            store = store,
            pdf = pdf.editor,
            ocr = pdf.ocr,
            texts = adapters.texts,
            versions = versions,
            activity = activity,
        )
        return PdfServices(
            editPages = EditPages(store = store, pdf = pdf.editor, versions = versions, activity = activity),
            splitDocument = SplitDocument(
                store = store,
                pdf = pdf.editor,
                replicator = replicator,
                names = names,
                trashDocument = trashDocument,
                activity = activity,
                clock = clock,
                ids = ids,
            ),
            mergeDocuments = MergeDocuments(
                store = store,
                pdf = pdf.editor,
                replicator = replicator,
                names = names,
                trashDocument = trashDocument,
                activity = activity,
                clock = clock,
                ids = ids,
            ),
            makeSearchable = makeSearchable,
            makeAllSearchable = MakeAllSearchable(store = store, makeSearchable = makeSearchable),
            scansWithoutText = ScansWithoutText(store = store, pdf = pdf.editor),
            embedPdfMetadata = EmbedPdfMetadata(store = store, pdf = pdf.editor, versions = versions),
            editor = pdf.editor,
            ocr = pdf.ocr,
        )
    }

    private fun askServices(): AskServices {
        val texts = DocumentTexts(reader = ai.reader, texts = adapters.texts)
        val vectors = VectorSearch(index = adapters.index)
        return AskServices(
            indexLibrary = IndexLibrary(
                store = store,
                texts = texts,
                embeddings = ai.embeddings,
                index = adapters.index,
                replicator = replicator,
            ),
            askLibrary = AskLibrary(
                store = store,
                search = matcher,
                texts = texts,
                embeddings = ai.embeddings,
                vectors = vectors,
                assistant = ai.libraryAssistant,
            ),
            relatedDocuments = RelatedDocuments(store = store, vectors = vectors),
        )
    }
}
