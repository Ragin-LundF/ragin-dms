package dms.ui.document

import androidx.lifecycle.ViewModel
import dms.application.catalog.model.CatalogEdit
import dms.application.common.OperationOutcome
import dms.domain.document.Document
import dms.domain.document.DocumentMetadata
import dms.domain.library.RelativePath
import dms.domain.taxonomy.CategoryId
import dms.domain.taxonomy.TagName
import dms.ui.PlatformActions
import dms.ui.components.DialogController
import dms.ui.components.model.ConfirmationRequest
import dms.ui.components.model.TextInputRequest
import dms.ui.document.model.MetadataDraft
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.NameInput
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.UiMessage
import java.time.LocalDate

/** Changes to the one document shown in the inspector: metadata, category, tags, name, place. */
class InspectorViewModel(
    private val services: AppServices,
    private val session: LibrarySession,
    private val platform: PlatformActions,
) : ViewModel() {

    val dialogs: DialogController = DialogController()

    val fileManagerName: String
        get() {
            return platform.fileManagerName
        }

    fun saveMetadata(document: Document, draft: MetadataDraft) {
        val date = draft.documentDate.trim()
            .takeIf { it.isNotEmpty() }
            ?.let { runCatching { LocalDate.parse(it) }.getOrNull() }
        if (draft.documentDate.isNotBlank() && date == null) {
            session.report(message = UiMessage(text = "The date must look like 2024-01-31", isError = true))
            return
        }
        val metadata = DocumentMetadata(
            title = draft.title.trim().ifEmpty { null },
            documentDate = date,
            correspondent = draft.correspondent.trim().ifEmpty { null },
        )
        edit(edit = CatalogEdit.UpdateMetadata(documentId = document.id, metadata = metadata))
    }

    fun assignCategory(document: Document, categoryId: CategoryId?) {
        edit(edit = CatalogEdit.AssignCategory(documentId = document.id, categoryId = categoryId))
    }

    fun addTag(document: Document, raw: String) {
        if (raw.isBlank()) {
            return
        }
        val tag = runCatching { TagName.of(raw = raw) }.getOrNull()
        if (tag == null) {
            session.report(
                message = UiMessage(text = "A tag has at most ${TagName.MAX_LENGTH} characters", isError = true),
            )
            return
        }
        edit(edit = CatalogEdit.AddTags(documentId = document.id, tags = setOf(tag)))
    }

    fun removeTag(document: Document, tag: TagName) {
        edit(edit = CatalogEdit.RemoveTags(documentId = document.id, tags = setOf(tag)))
    }

    /** Renames the file in every library folder; the original name stays until the user changes it. */
    fun rename(document: Document) {
        askName(title = "Rename file", label = "Name", initial = document.path.fileName) { name ->
            move(
                document = document,
                target = NameInput.child(parent = document.path.parent ?: RelativePath.ROOT, raw = name),
            )
        }
    }

    fun moveToFolder(document: Document) {
        val request = TextInputRequest(
            title = "Move ${document.path.fileName}",
            label = "Folder, such as Letters/2024, or / for the top level",
            initialValue = document.path.parent?.value?.ifEmpty { "/" } ?: "/",
            confirmLabel = "Move",
        )
        dialogs.askText(request = request) { raw ->
            val folder = raw.trim().trim('/', '\\')
            val parent = runCatching { RelativePath.of(raw = folder) }.getOrNull()
            val problem = folder.takeIf { it.isNotEmpty() }
                ?.split('/')
                ?.firstNotNullOfOrNull { NameInput.problem(raw = it) }
            if (parent == null || problem != null) {
                session.report(message = UiMessage(text = problem ?: "Not a valid folder", isError = true))
            } else {
                move(document = document, target = parent.resolve(name = document.path.fileName))
            }
        }
    }

    fun trash(document: Document) {
        val folders = session.state.value.folders?.folders.orEmpty()
        dialogs.ask(
            request = ConfirmationRequest(
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

    fun showInFileManager(document: Document) {
        val location = locationOf(document = document) ?: return
        if (!platform.showInFileManager(location = location)) {
            session.report(message = UiMessage(text = "Could not open the ${platform.fileManagerName}", isError = true))
        }
    }

    fun open(document: Document) {
        val location = locationOf(document = document) ?: return
        if (!platform.openFile(location = location)) {
            session.report(
                message = UiMessage(text = "No application could open ${document.path.fileName}", isError = true),
            )
        }
    }

    private fun move(document: Document, target: RelativePath) {
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
            request = TextInputRequest(title = title, label = label, initialValue = initial, confirmLabel = "Rename"),
        ) { name ->
            val problem = NameInput.problem(raw = name)
            if (problem == null) {
                submit(name)
            } else {
                session.report(message = UiMessage(text = problem, isError = true))
            }
        }
    }

    /** The file in the primary folder, where every document lives. */
    private fun locationOf(document: Document): String? {
        val primary = session.state.value.folders?.primary ?: return null
        return platform.locate(folder = primary.location, path = document.path)
    }
}
