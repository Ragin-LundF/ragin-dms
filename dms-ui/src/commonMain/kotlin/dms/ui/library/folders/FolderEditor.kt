package dms.ui.library.folders

import dms.application.common.OperationOutcome
import dms.application.folder.model.FolderEntriesOutcome
import dms.application.port.storage.model.FolderEntriesModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.ui.components.dialog.DialogController
import dms.ui.components.model.ConfirmationRequestModel
import dms.ui.library.folders.model.FolderNaming
import dms.ui.library.folders.model.FolderViewStateModel
import dms.ui.library.picker.FolderPicker
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.NameInput
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.UiMessageModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/**
 * Creates, renames, moves and deletes folders in every library folder, through the session's use cases.
 * Names are typed in place: [newFolder] and [rename] open a name, [commitName] saves it, [cancelName]
 * drops it. The view state follows every change, so the open folder and the tree stay where they were.
 */
class FolderEditor(
    private val services: AppServices,
    private val session: LibrarySession,
    private val dialogs: DialogController,
    private val view: MutableStateFlow<FolderViewStateModel>,
    private val documents: () -> Collection<DocumentModel>,
    private val foldersOnDisk: () -> Set<RelativePathModel>,
    private val onFoldersChanged: () -> Unit,
    private val scope: CoroutineScope,
) {

    /** The "Move to…" picker of the folder view. */
    val picker: FolderPicker = FolderPicker()

    /** Opens [parent] with a new folder whose name is typed in place. */
    fun newFolder(parent: RelativePathModel) {
        saveName()
        val proposed = FolderContents.proposedName(parent = parent, folders = knownFolders())
        view.update { state ->
            state.copy(
                open = parent,
                naming = FolderNaming.New(parent = parent, proposed = proposed),
                draft = proposed,
                expanded = state.expanded + parent,
            )
        }
    }

    /** Opens the folder around [path] with [path]'s name ready to be typed over; the top level has no name. */
    fun rename(path: RelativePathModel) {
        if (path.isRoot) {
            session.report(message = UiMessageModel(text = "The library's top level cannot be renamed", isError = true))
            return
        }
        saveName()
        val naming = FolderNaming.Rename(path = path)
        view.update { it.copy(open = naming.parent, naming = naming, draft = path.fileName) }
    }

    /** Keeps what the user typed so far, so clicking elsewhere in the view can save it ([saveName]). */
    fun type(text: String) {
        view.update { it.copy(draft = text) }
    }

    /** Saves the name being typed, if any, as it reads now; clicking away from the name does this. */
    fun saveName() {
        val naming = view.value.naming ?: return
        commitName(naming = naming, text = view.value.draft)
    }

    /**
     * Saves [text] as the name [naming] asked for; nothing happens when that name is no longer being
     * typed, so Enter followed by losing the focus saves once. A blank or unchanged name changes nothing.
     */
    fun commitName(naming: FolderNaming, text: String) {
        if (view.value.naming != naming) {
            return
        }
        view.update { it.copy(naming = null) }
        val name = text.trim().replace(oldChar = '/', newChar = '-')
        if (name.isEmpty() || (naming is FolderNaming.Rename && name == naming.path.fileName)) {
            return
        }
        val problem = NameInput.problem(raw = name)
        if (problem != null) {
            session.report(message = UiMessageModel(text = problem, isError = true))
            return
        }
        val target = NameInput.child(parent = naming.parent, raw = name)
        when (naming) {
            is FolderNaming.New -> create(path = target)
            is FolderNaming.Rename -> move(
                from = naming.path,
                to = target,
                done = "Renamed to \"$name\" in every library folder",
            )
        }
    }

    fun cancelName() {
        view.update { it.copy(naming = null) }
    }

    /** Opens the picker to move [path]; the folder itself and its subfolders cannot be picked. */
    fun chooseTarget(path: RelativePathModel) {
        saveName()
        picker.open(title = "Move \"${path.fileName}\" to…", excluded = path) { target ->
            moveInto(path = path, folder = target)
        }
    }

    /**
     * Moves [path] into [folder]. Into itself or one of its subfolders is refused with a message; a move
     * to where it already is does nothing.
     */
    fun moveInto(path: RelativePathModel, folder: RelativePathModel) {
        if (FolderContents.isAtOrBelow(path = folder, folder = path)) {
            session.report(
                message = UiMessageModel(
                    text = "A folder cannot be moved into itself or one of its subfolders",
                    isError = true,
                ),
            )
            return
        }
        val target = folder.resolve(name = path.fileName)
        if (target == path) {
            return
        }
        view.update { it.copy(expanded = it.expanded + folder) }
        move(from = path, to = target, done = "Moved \"${path.fileName}\" to ${FolderTexts.where(folder = folder)}")
    }

    /**
     * Reads what really lies in [path] in every library folder, then asks, naming the documents, the other
     * files the library does not know (new, or only in a sync folder) and the subfolders that go along,
     * and trashes [path] everywhere. The top level cannot be deleted.
     */
    fun delete(path: RelativePathModel) {
        if (path.isRoot) {
            session.report(message = UiMessageModel(text = "The library's top level cannot be deleted", isError = true))
            return
        }
        saveName()
        scope.launch {
            // Without knowing what is inside, the confirmation could not name it: nothing is offered then.
            when (val outcome = services.documents.readFolderEntries(path = path)) {
                is FolderEntriesOutcome.Read -> ask(path = path, entries = outcome.entries)
                FolderEntriesOutcome.LibraryNotOpen -> session.report(
                    message = OutcomeMessages.of(outcome = OperationOutcome.LibraryNotOpen, done = ""),
                )

                is FolderEntriesOutcome.StorageFailed -> session.report(
                    message = UiMessageModel(
                        text = FolderTexts.unreadable(folder = path, reason = outcome.failure.reason),
                        isError = true,
                    ),
                )
            }
        }
    }

    private fun ask(path: RelativePathModel, entries: FolderEntriesModel) {
        val inside = documents().filter { it.path.isInside(folder = path) }
        val catalogued = inside.map { it.path }.toSet()
        val others = (entries.files - catalogued).sortedBy { it.value }
        val subfolders = (entries.folders + knownFolders().filter { it.isInside(folder = path) }).size
        val request = ConfirmationRequestModel(
            title = FolderTexts.deleteTitle(folder = path),
            message = FolderTexts.deleteMessage(
                documents = inside.size,
                otherFiles = others.size,
                subfolders = subfolders,
            ),
            affected = inside.map { it.path.value } + others.map { "${it.value} (not in the library)" },
            confirmLabel = "Delete folder",
        )
        dialogs.ask(request = request) { trash(path = path, documents = inside.size + others.size) }
    }

    private fun create(path: RelativePathModel) {
        run(label = "Creating folder ${path.value}") {
            val outcome = services.documents.createFolder(path = path)
            if (outcome is OperationOutcome.Done) {
                onFoldersChanged()
            }
            session.report(message = OutcomeMessages.of(outcome = outcome, done = "Created \"${path.fileName}\""))
        }
    }

    private fun move(from: RelativePathModel, to: RelativePathModel, done: String) {
        run(label = "Moving folder ${from.value}") {
            val outcome = services.documents.moveFolder(from = from, to = to)
            if (outcome is OperationOutcome.Done) {
                view.update { state ->
                    state.copy(
                        open = FolderContents.relocated(path = state.open, from = from, to = to),
                        expanded = FolderContents.relocated(paths = state.expanded, from = from, to = to),
                    )
                }
                onFoldersChanged()
            }
            session.report(message = OutcomeMessages.of(outcome = outcome, done = done))
        }
    }

    private fun trash(path: RelativePathModel, documents: Int) {
        run(label = "Deleting folder ${path.value}") {
            val outcome = services.documents.trashFolder(path = path, confirmed = true)
            if (outcome is OperationOutcome.Done) {
                view.update { state ->
                    val gone = { folder: RelativePathModel -> FolderContents.isAtOrBelow(path = folder, folder = path) }
                    state.copy(
                        open = if (gone(state.open)) path.parent ?: RelativePathModel.ROOT else state.open,
                        expanded = state.expanded.filterNot(gone).toSet(),
                    )
                }
                onFoldersChanged()
            }
            val done = if (documents == 0) {
                "Deleted \"${path.fileName}\""
            } else {
                "Deleted \"${path.fileName}\"; its documents are in the trash"
            }
            session.report(message = OutcomeMessages.of(outcome = outcome, done = done))
        }
    }

    /** Runs [block] as a library operation, or says what it waits for while another one runs. */
    private fun run(label: String, block: suspend () -> Unit) {
        if (!session.perform(label = label, block = block)) {
            val busy = session.state.value.busy.orEmpty()
            session.report(message = UiMessageModel(text = OutcomeMessages.waitFor(operation = busy), isError = true))
        }
    }

    private fun knownFolders(): List<RelativePathModel> {
        return FolderContents.knownFolders(documents = documents(), foldersOnDisk = foldersOnDisk())
    }
}
