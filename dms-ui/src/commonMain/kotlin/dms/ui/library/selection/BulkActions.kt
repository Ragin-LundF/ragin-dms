package dms.ui.library.selection

import dms.application.catalog.model.CatalogEdit
import dms.application.classification.model.SuggestOutcome
import dms.application.common.OperationOutcome
import dms.application.port.ai.model.ClassificationFailure
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.components.dialog.DialogController
import dms.ui.components.model.ConfirmationRequestModel
import dms.ui.components.model.TextInputRequestModel
import dms.ui.library.LibraryTrees
import dms.ui.library.model.FolderEntryModel
import dms.ui.library.picker.FolderPicker
import dms.ui.review.AiMessages
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.model.OperationLane
import dms.ui.session.model.OperationProgressModel
import dms.ui.session.model.UiMessageModel

/**
 * What the bulk bar does with the ticked documents ([chosen]): set a category, add a tag, move them
 * through the folder [picker], let the AI suggest tags, or move them to the trash after asking. Every
 * action runs the use cases one document after the other through [LibrarySession.perform] and ends the
 * selection with [done].
 */
class BulkActions(
    private val services: AppServices,
    private val session: LibrarySession,
    private val dialogs: DialogController,
    private val chosen: () -> List<DocumentModel>,
    private val foldersOnDisk: () -> Set<RelativePathModel>,
    private val done: () -> Unit,
) {

    val picker: FolderPicker = FolderPicker()

    /** Files the documents under [categoryId], or removes their category with `null`. */
    fun assignCategory(categoryId: CategoryIdModel?, documents: List<DocumentModel> = chosen()) {
        val changing = documents.filter { it.categoryId != categoryId }
        val name = categoryId?.let { services.library.catalog.value?.categories?.get(it)?.name }
        val step = if (name == null) "Removing the category" else "Filing under $name"
        edit(documents = changing, step = step) {
            CatalogEdit.AssignCategory(documentId = it.id, categoryId = categoryId)
        }
    }

    fun addTag(tag: TagNameModel, documents: List<DocumentModel> = chosen()) {
        val changing = documents.filter { tag !in it.tags }
        edit(documents = changing, step = "Tagging with ${tag.value}") {
            CatalogEdit.AddTags(documentId = it.id, tags = setOf(tag))
        }
    }

    /** Asks for a tag, new or known, and adds it to the documents. */
    fun newTag() {
        val count = chosen().size
        val request = TextInputRequestModel(
            title = "Tag ${documentsText(count = count)}",
            label = "Tag",
            initialValue = "",
            confirmLabel = "Tag",
        )
        dialogs.askText(request = request) { raw ->
            val tag = TagNameModel.parse(raw = raw)
            if (tag == null) {
                session.report(message = UiMessageModel(text = "A tag needs a name", isError = true))
            } else {
                addTag(tag = tag)
            }
        }
    }

    /** Opens the folder picker and moves the documents into the folder picked, in every library folder. */
    fun move() {
        val documents = chosen()
        picker.open(title = "Move ${documentsText(count = documents.size)} to…") { target ->
            moveTo(target = target, documents = documents)
        }
    }

    /** Moves [documents] into [target] in every library folder; one already there stays where it is. */
    fun moveTo(target: RelativePathModel, documents: List<DocumentModel> = chosen()) {
        val moving = documents.filter { (it.path.parent ?: RelativePathModel.ROOT) != target }
        val where = target.value.ifEmpty { "the top level" }
        run(documents = moving, step = "Moving to $where") { document ->
            val outcome = services.documents.moveDocument(
                documentId = document.id,
                newPath = target.resolve(name = document.path.fileName),
            )
            outcome is OperationOutcome.Done
        }
    }

    /** Lets the AI suggest tags for every document, in the AI lane; the suggestions wait under Needs review. */
    fun tagWithAi() {
        val documents = chosen()
        val step = "Tagging ${documentsText(count = documents.size)} with AI"
        val started = session.perform(label = step, lane = OperationLane.AI, cancellable = true) {
            var suggested = 0
            for ((index, document) in documents.withIndex()) {
                val progress = OperationProgressModel(step = step, done = index, total = documents.size)
                session.progress(progress = progress.copy(detail = document.path.value), lane = OperationLane.AI)
                when (val outcome = services.ai.suggestTags(documentId = document.id)) {
                    is SuggestOutcome.Suggested -> if (outcome.suggestion.tags.isNotEmpty()) {
                        suggested++
                        session.changed()
                    }

                    is SuggestOutcome.Failed -> if (outcome.reason == ClassificationFailure.PROVIDER_UNAVAILABLE) {
                        session.report(message = AiMessages.of(outcome = outcome, name = document.path.fileName))
                        return@perform
                    }

                    SuggestOutcome.LibraryNotOpen,
                    SuggestOutcome.NotFound,
                    is SuggestOutcome.NotReadable,
                    -> session.log.error(text = AiMessages.of(outcome = outcome, name = document.path.fileName).text)
                }
            }
            session.report(
                message = UiMessageModel(
                    text = "The AI suggested tags for $suggested of ${documents.size} document(s); " +
                        "review them under Needs review",
                ),
            )
        }
        if (started) {
            done()
        }
    }

    /** Asks first, naming every document, then moves them to the system trash from every library folder. */
    fun trash() {
        val documents = chosen()
        val request = ConfirmationRequestModel(
            title = "Move ${documentsText(count = documents.size)} to the trash?",
            message = "They are moved to the system trash from every library folder; " +
                "you can restore them from there.",
            affected = documents.map { it.path.value },
            confirmLabel = "Move to trash",
        )
        dialogs.ask(request = request) {
            run(documents = documents, step = "Moving to the trash") { document ->
                services.documents.trashDocument(documentId = document.id, confirmed = true) is OperationOutcome.Done
            }
        }
    }

    /**
     * The rows of the folder picker below its "Library" row: every folder, also the empty ones on disk,
     * subfolders while [expanded].
     */
    fun pickerFolders(expanded: Set<RelativePathModel>): List<FolderEntryModel> {
        val documents = services.library.catalog.value?.documents?.values.orEmpty()
        return LibraryTrees.folders(documents = documents, foldersOnDisk = foldersOnDisk(), expanded = expanded)
            .filterNot { it.path.isRoot }
    }

    private fun edit(documents: List<DocumentModel>, step: String, editOf: (DocumentModel) -> CatalogEdit) {
        run(documents = documents, step = step) { document ->
            services.documents.editCatalog(edit = editOf(document)) is OperationOutcome.Done
        }
    }

    /** Runs [action] for every document as one library operation and says how many succeeded. */
    private fun run(documents: List<DocumentModel>, step: String, action: suspend (DocumentModel) -> Boolean) {
        if (documents.isEmpty()) {
            done()
            return
        }
        val started = session.perform(label = "$step: ${documentsText(count = documents.size)}") {
            val succeeded = session.forEachWithProgress(
                step = step,
                items = documents,
                name = { it.path.value },
                action = action,
            )
            session.report(
                message = UiMessageModel(
                    text = "$step: $succeeded of ${documentsText(count = documents.size)} done",
                    isError = succeeded < documents.size,
                ),
            )
        }
        if (started) {
            done()
        }
    }

    private fun documentsText(count: Int): String {
        return if (count == 1) "1 document" else "$count documents"
    }
}
