package dms.ui.document

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dms.application.catalog.model.CatalogEdit
import dms.application.classification.model.SuggestOutcome
import dms.application.common.OperationOutcome
import dms.application.port.state.model.ActivityEntryModel
import dms.domain.catalog.model.CatalogModel
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.document.DocumentKind
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.PlatformActions
import dms.ui.assistant.ChatController
import dms.ui.assistant.SummaryController
import dms.ui.components.dialog.DialogController
import dms.ui.components.model.ConfirmationRequestModel
import dms.ui.components.model.TextInputRequestModel
import dms.ui.document.details.DetailsController
import dms.ui.document.model.DocumentFactModel
import dms.ui.document.model.HistoryEntryModel
import dms.ui.document.model.InspectorTab
import dms.ui.document.model.MetadataDraftModel
import dms.ui.document.suggestion.DashboardFeed
import dms.ui.document.tab.DocumentFacts
import dms.ui.document.tab.DocumentHistory
import dms.ui.pdf.PageEditorController
import dms.ui.pdf.PdfActions
import dms.ui.preview.ThumbnailLoader
import dms.ui.review.AiMessages
import dms.ui.review.ReviewController
import dms.ui.service.AppServices
import dms.ui.session.DateInput
import dms.ui.session.LibrarySession
import dms.ui.session.NameInput
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.OperationLane
import dms.ui.session.model.UiMessageModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import java.time.ZoneId

/**
 * Changes to the one document shown in the inspector: metadata, category, tags, name, place; what
 * the AI can do with it: summarize, tag, classify, and answer questions; its pending suggestion; and
 * which tab of the inspector shows.
 */
class InspectorViewModel(
    private val services: AppServices,
    private val session: LibrarySession,
    private val platform: PlatformActions,
) : ViewModel() {

    val dialogs: DialogController = DialogController()
    val review: ReviewController = ReviewController(services = services, session = session)
    val summary: SummaryController = SummaryController(services = services, scope = viewModelScope)
    val chat: ChatController = ChatController(services = services, scope = viewModelScope)
    val details: DetailsController = DetailsController(services = services, session = session)
    val pages: PageEditorController = PageEditorController(
        services = services,
        session = session,
        platform = platform,
        dialogs = dialogs,
        scope = viewModelScope,
    )
    val pdf: PdfActions = PdfActions(services = services, session = session, dialogs = dialogs, scope = viewModelScope)

    val thumbnails: ThumbnailLoader = ThumbnailLoader(services = services, platform = platform)

    /** The suggestions that wait for review and the recent activity, as the dashboard has them. */
    val feed: DashboardFeed = DashboardFeed(services = services, session = session, scope = viewModelScope)

    private val selectedTab = MutableStateFlow(value = InspectorTab.DETAILS)

    /** The tab the user chose; it stays when another document is inspected. */
    val tab: StateFlow<InspectorTab> = selectedTab.asStateFlow()

    /** The open catalog, for the sections that show more than the document itself. */
    val catalog: StateFlow<CatalogModel?> = services.library.catalog

    val fileManagerName: String
        get() {
            return platform.fileManagerName
        }

    fun selectTab(tab: InspectorTab) {
        selectedTab.value = tab
    }

    /** The tab to show for [document]: the [selected] one, or Details if it does not apply to [document]. */
    fun shownTab(selected: InspectorTab, document: DocumentModel): InspectorTab {
        return if (selected in disabledTabs(document = document)) InspectorTab.DETAILS else selected
    }

    /** The tabs that do not apply to [document], with the reason. */
    fun disabledTabs(document: DocumentModel): Map<InspectorTab, String> {
        if (document.kind == DocumentKind.PDF) {
            return emptyMap()
        }
        return mapOf(InspectorTab.PAGES to "Only PDFs have pages")
    }

    fun factsOf(document: DocumentModel, catalog: CatalogModel): List<DocumentFactModel> {
        return DocumentFacts.of(
            document = document,
            catalog = catalog,
            today = services.library.clock.today(),
            zone = ZoneId.systemDefault(),
        )
    }

    fun historyOf(document: DocumentModel, activity: List<ActivityEntryModel>): List<HistoryEntryModel> {
        return DocumentHistory.of(document = document, activity = activity)
    }

    /** Opens the review of a suggestion that waits for this document. */
    fun reviewSuggestion(suggestion: ClassificationSuggestionModel) {
        review.open(suggestion = suggestion)
    }

    /** Rejects the suggestion for [document], as "Reject" on the dashboard does; nothing else changes. */
    fun dismissSuggestion(document: DocumentModel) {
        session.perform(label = "Rejecting the suggestion for ${document.path.fileName}") {
            services.ai.rejectSuggestion(documentId = document.id)
        }
    }

    fun saveMetadata(document: DocumentModel, draft: MetadataDraftModel) {
        val date = draft.documentDate.takeIf { it.isNotBlank() }?.let { DateInput.parse(raw = it) }
        if (draft.documentDate.isNotBlank() && date == null) {
            session.report(message = UiMessageModel(text = DateInput.PROBLEM, isError = true))
            return
        }
        val metadata = DocumentMetadataModel(
            title = draft.title.trim().ifEmpty { null },
            documentDate = date,
            correspondent = draft.correspondent.trim().ifEmpty { null },
        )
        edit(edit = CatalogEdit.UpdateMetadata(documentId = document.id, metadata = metadata))
    }

    fun assignCategory(document: DocumentModel, categoryId: CategoryIdModel?) {
        edit(edit = CatalogEdit.AssignCategory(documentId = document.id, categoryId = categoryId))
    }

    fun addTag(document: DocumentModel, raw: String) {
        if (raw.isBlank()) {
            return
        }
        val tag = TagNameModel.parse(raw = raw)
        if (tag == null) {
            session.report(
                message = UiMessageModel(
                    text = "A tag has at most ${TagNameModel.MAX_LENGTH} characters",
                    isError = true,
                ),
            )
            return
        }
        edit(edit = CatalogEdit.AddTags(documentId = document.id, tags = setOf(tag)))
    }

    fun removeTag(document: DocumentModel, tag: TagNameModel) {
        edit(edit = CatalogEdit.RemoveTags(documentId = document.id, tags = setOf(tag)))
    }

    /** Lets the AI suggest a classification and opens it for review right away. */
    fun classifyWithAi(document: DocumentModel) {
        val name = document.path.fileName
        session.perform(label = "Classifying $name with AI", lane = OperationLane.AI, cancellable = true) {
            when (val outcome = services.ai.suggestClassification(documentId = document.id)) {
                is SuggestOutcome.Suggested -> review.open(suggestion = outcome.suggestion)
                SuggestOutcome.LibraryNotOpen,
                SuggestOutcome.NotFound,
                is SuggestOutcome.NotReadable,
                is SuggestOutcome.Failed,
                -> session.report(message = AiMessages.of(outcome = outcome, name = name))
            }
        }
    }

    /** Lets the AI suggest tags and opens them for review right away. */
    fun tagWithAi(document: DocumentModel) {
        val name = document.path.fileName
        session.perform(label = "Tagging $name with AI", lane = OperationLane.AI, cancellable = true) {
            when (val outcome = services.ai.suggestTags(documentId = document.id)) {
                is SuggestOutcome.Suggested -> if (outcome.suggestion.tags.isEmpty()) {
                    session.report(message = UiMessageModel(text = "The AI found no new tags for $name"))
                } else {
                    review.open(suggestion = outcome.suggestion)
                }

                SuggestOutcome.LibraryNotOpen,
                SuggestOutcome.NotFound,
                is SuggestOutcome.NotReadable,
                is SuggestOutcome.Failed,
                -> session.report(message = AiMessages.of(outcome = outcome, name = name))
            }
        }
    }

    /** Renames the file in every library folder; the original name stays until the user changes it. */
    fun rename(document: DocumentModel) {
        askName(title = "Rename file", label = "Name", initial = document.path.fileName) { name ->
            move(
                document = document,
                target = NameInput.child(parent = document.path.parent ?: RelativePathModel.ROOT, raw = name),
            )
        }
    }

    fun moveToFolder(document: DocumentModel) {
        val request = TextInputRequestModel(
            title = "Move ${document.path.fileName}",
            label = "Folder, such as Letters/2024, or / for the top level",
            initialValue = document.path.parent?.value?.ifEmpty { "/" } ?: "/",
            confirmLabel = "Move",
        )
        dialogs.askText(request = request) { raw ->
            val folder = raw.trim().trim('/', '\\')
            val parent = RelativePathModel.parse(raw = folder)
            val problem = folder.takeIf { it.isNotEmpty() }
                ?.split('/')
                ?.firstNotNullOfOrNull { NameInput.problem(raw = it) }
            if (parent == null || problem != null) {
                session.report(message = UiMessageModel(text = problem ?: "Not a valid folder", isError = true))
            } else {
                move(document = document, target = parent.resolve(name = document.path.fileName))
            }
        }
    }

    fun trash(document: DocumentModel) {
        val folders = session.state.value.folders?.folders.orEmpty()
        dialogs.ask(
            request = ConfirmationRequestModel(
                title = "Move to trash",
                message = "The file goes to the trash in every library folder and leaves the catalog.",
                affected = folders.map { "${it.displayName}: ${document.path}" },
                confirmLabel = "Move to trash",
            ),
        ) {
            session.perform(label = "Moving ${document.path} to the trash") {
                val outcome = services.documents.trashDocument(documentId = document.id, confirmed = true)
                session.report(
                    message = OutcomeMessages.of(outcome = outcome, done = "Moved ${document.path} to the trash"),
                )
            }
        }
    }

    fun showInFileManager(document: DocumentModel) {
        val location = locationOf(document = document) ?: return
        if (!platform.showInFileManager(location = location)) {
            session.report(
                message = UiMessageModel(text = "Could not open the ${platform.fileManagerName}", isError = true),
            )
        }
    }

    fun open(document: DocumentModel) {
        val location = locationOf(document = document) ?: return
        if (!platform.openFile(location = location)) {
            session.report(
                message = UiMessageModel(text = "No application could open ${document.path.fileName}", isError = true),
            )
        }
    }

    /** Opens a catalogued document by its id, for example from the review of a suggestion. */
    fun open(documentId: DocumentIdModel) {
        services.library.catalog.value?.documents?.get(documentId)?.let { open(document = it) }
    }

    private fun move(document: DocumentModel, target: RelativePathModel) {
        session.perform(label = "Moving ${document.path}") {
            val outcome = services.documents.moveDocument(documentId = document.id, newPath = target)
            session.report(message = OutcomeMessages.of(outcome = outcome, done = "Moved to ${target.value}"))
        }
    }

    private fun edit(edit: CatalogEdit) {
        session.perform(label = "Saving the catalog") {
            val outcome = services.documents.editCatalog(edit = edit)
            if (outcome !is OperationOutcome.Done) {
                session.report(message = OutcomeMessages.of(outcome = outcome, done = ""))
            }
        }
    }

    private fun askName(title: String, label: String, initial: String, submit: (String) -> Unit) {
        dialogs.askText(
            request = TextInputRequestModel(
                title = title,
                label = label,
                initialValue = initial,
                confirmLabel = "Rename",
            ),
        ) { name ->
            val problem = NameInput.problem(raw = name)
            if (problem == null) {
                submit(name)
            } else {
                session.report(message = UiMessageModel(text = problem, isError = true))
            }
        }
    }

    /** The file in the primary folder, where every document lives. */
    private fun locationOf(document: DocumentModel): String? {
        val primary = session.state.value.folders?.primary ?: return null
        return platform.locate(folder = primary.location, path = document.path)
    }
}
