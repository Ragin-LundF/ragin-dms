package dms.ui.document

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dms.application.catalog.model.CatalogEdit
import dms.application.classification.model.SuggestOutcome
import dms.application.common.OperationOutcome
import dms.domain.catalog.model.CatalogModel
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.document.model.DocumentModel
import dms.domain.field.model.FieldDefinitionModel
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
import dms.ui.document.model.MetadataDraftModel
import dms.ui.document.model.QuickFact
import dms.ui.document.model.QuickFactsModel
import dms.ui.document.quicklook.QuickFacts
import dms.ui.document.quicklook.QuickLookController
import dms.ui.document.suggestion.DashboardFeed
import dms.ui.document.suggestion.SuggestionValues
import dms.ui.pdf.PageEditorController
import dms.ui.pdf.PdfActions
import dms.ui.pdf.rebuild.RebuildController
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
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn

/**
 * Changes to the one document shown in the quick look: metadata, category, tags, fields, name, place;
 * what the AI can do with it: summarize, tag, classify, and answer questions; and its pending
 * suggestion. [quickLook] holds which document shows and the fact edited in place.
 */
class QuickLookViewModel(
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
    val rebuild: RebuildController = RebuildController(
        services = services,
        session = session,
        platform = platform,
        dialogs = dialogs,
        scope = viewModelScope,
    )

    val thumbnails: ThumbnailLoader = ThumbnailLoader(services = services, platform = platform)

    /** The suggestions that wait for review and the recent activity, as the dashboard has them. */
    val feed: DashboardFeed = DashboardFeed(session = session, scope = viewModelScope)

    val quickLook: QuickLookController = QuickLookController()

    /** Whether AI is set up; without it the quick look offers no AI action. */
    val aiEnabled: StateFlow<Boolean> = session.state
        .map { it.aiEnabled }
        .stateIn(scope = viewModelScope, started = SharingStarted.Eagerly, initialValue = session.state.value.aiEnabled)

    /** The open catalog, for the sections that show more than the document itself. */
    val catalog: StateFlow<CatalogModel?> = services.library.catalog

    val fileManagerName: String
        get() {
            return platform.fileManagerName
        }

    fun factsOf(document: DocumentModel, catalog: CatalogModel): QuickFactsModel {
        return QuickFacts.of(document = document, catalog = catalog, today = services.library.clock.today())
    }

    /** The values of [suggestion] that "Accept all" applies, as chips. */
    fun suggestedValues(suggestion: ClassificationSuggestionModel, catalog: CatalogModel): List<String> {
        return SuggestionValues.of(
            suggestion = suggestion,
            catalog = catalog,
            acceptFileName = session.state.value.preferences.ai.acceptSuggestedFileNames,
        )
    }

    /** Opens the review of a suggestion that waits for this document. */
    fun reviewSuggestion(suggestion: ClassificationSuggestionModel) {
        review.open(suggestion = suggestion)
    }

    /**
     * Applies [suggestion] unchanged through the same approval as the review dialog, so exactly the
     * values of [suggestedValues]. If the approval is refused before anything changes, the review
     * opens with the reason.
     */
    fun acceptSuggestion(suggestion: ClassificationSuggestionModel) {
        review.open(suggestion = suggestion)
        review.approve()
    }

    /** Rejects the suggestion for [document], as "Reject" on the dashboard does; nothing else changes. */
    fun dismissSuggestion(document: DocumentModel) {
        session.perform(label = "Rejecting the suggestion for ${document.path.fileName}") {
            services.ai.rejectSuggestion(documentId = document.id)
        }
    }

    /** Saves title, date and correspondent together; `false` when the date is no date, which the message says. */
    fun saveMetadata(document: DocumentModel, draft: MetadataDraftModel): Boolean {
        val date = draft.documentDate.takeIf { it.isNotBlank() }?.let { DateInput.parse(raw = it) }
        if (draft.documentDate.isNotBlank() && date == null) {
            session.report(message = UiMessageModel(text = DateInput.PROBLEM, isError = true))
            return false
        }
        val metadata = DocumentMetadataModel(
            title = draft.title.trim().ifEmpty { null },
            documentDate = date,
            correspondent = draft.correspondent.trim().ifEmpty { null },
        )
        edit(edit = CatalogEdit.UpdateMetadata(documentId = document.id, metadata = metadata))
        return true
    }

    /**
     * Saves what the user typed in place for [fact] and ends the edit; a value that does not fit is
     * refused with a message, and the edit stays open to fix it.
     */
    fun saveFact(document: DocumentModel, fact: QuickFact, typed: String) {
        val catalog = services.library.catalog.value ?: return
        val current = MetadataDraftModel(
            title = document.metadata.title.orEmpty(),
            documentDate = document.metadata.documentDate?.toString().orEmpty(),
            correspondent = document.metadata.correspondent.orEmpty(),
        )
        val amount = QuickFacts.amountField(catalog = catalog)
        val due = QuickFacts.dueField(catalog = catalog)
        val saved = when (fact) {
            QuickFact.TITLE -> saveMetadata(document = document, draft = current.copy(title = typed))
            QuickFact.CORRESPONDENT -> saveMetadata(document = document, draft = current.copy(correspondent = typed))
            QuickFact.DATE -> saveMetadata(document = document, draft = current.copy(documentDate = typed))
            QuickFact.AMOUNT -> saveField(document = document, field = amount, typed = typed)
            QuickFact.DUE -> saveField(document = document, field = due, typed = typed)
            QuickFact.TAG -> addTag(document = document, raw = typed)
        }
        if (saved) {
            quickLook.endEdit()
        }
    }

    fun assignCategory(document: DocumentModel, categoryId: CategoryIdModel?) {
        edit(edit = CatalogEdit.AssignCategory(documentId = document.id, categoryId = categoryId))
    }

    /** Adds the tag in [raw]; `false` when it is too long, which the message says. Nothing typed adds nothing. */
    fun addTag(document: DocumentModel, raw: String): Boolean {
        if (raw.isBlank()) {
            return true
        }
        val tag = TagNameModel.parse(raw = raw)
        if (tag == null) {
            session.report(
                message = UiMessageModel(
                    text = "A tag has at most ${TagNameModel.MAX_LENGTH} characters",
                    isError = true,
                ),
            )
            return false
        }
        edit(edit = CatalogEdit.AddTags(documentId = document.id, tags = setOf(tag)))
        return true
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

    private fun saveField(document: DocumentModel, field: FieldDefinitionModel?, typed: String): Boolean {
        return field != null && details.saveField(document = document, key = field.key, typed = typed)
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
