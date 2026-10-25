package dms.ui.library.taxonomy

import dms.application.catalog.model.CatalogEdit
import dms.application.common.OperationOutcome
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.components.dialog.DialogController
import dms.ui.components.model.ConfirmationRequestModel
import dms.ui.components.model.TextInputRequestModel
import dms.ui.library.model.LibrarySelection
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.UiMessageModel

/**
 * Creating, renaming, moving and deleting the categories and tags of the library, and saving catalog
 * edits for the library screen. [select] changes the sidebar selection after an edit.
 */
class TaxonomyController(
    private val services: AppServices,
    private val session: LibrarySession,
    private val dialogs: DialogController,
    private val select: (LibrarySelection) -> Unit,
) {

    /** A new category below [parentId], or at the top level. */
    fun newCategory(parentId: CategoryIdModel?) {
        askCategoryName(title = "New category", initial = "", confirm = "Create") { name ->
            edit(edit = CatalogEdit.AddCategory(name = name, parentId = parentId))
        }
    }

    fun renameCategory(category: CategoryModel) {
        askCategoryName(title = "Rename category", initial = category.name, confirm = "Rename") { name ->
            edit(edit = CatalogEdit.RenameCategory(categoryId = category.id, name = name))
        }
    }

    /** Moves [category] below [parentId], or to the top level. */
    fun moveCategory(category: CategoryModel, parentId: CategoryIdModel?) {
        val where = parentId?.let { services.library.catalog.value?.categories?.get(it)?.name } ?: "the top level"
        edit(
            edit = CatalogEdit.MoveCategory(categoryId = category.id, parentId = parentId),
            done = "Moved ${category.name} to $where",
        )
    }

    fun removeCategory(category: CategoryModel) {
        dialogs.ask(
            request = ConfirmationRequestModel(
                title = "Delete category \"${category.name}\"",
                message = "The category leaves the catalog. No document uses it and no file changes.",
                affected = emptyList(),
                confirmLabel = "Delete category",
            ),
        ) {
            edit(edit = CatalogEdit.RemoveCategory(categoryId = category.id)) {
                select(LibrarySelection.Folder(path = RelativePathModel.ROOT))
            }
        }
    }

    fun renameTag(tag: TagNameModel) {
        askText(title = "Rename tag", label = "Tag", initial = tag.value, confirm = "Rename") { name ->
            val renamed = TagNameModel.parse(raw = name)
            if (renamed == null) {
                session.report(message = UiMessageModel(text = "Not a valid tag", isError = true))
            } else {
                edit(edit = CatalogEdit.RenameTag(from = tag, to = renamed)) {
                    select(LibrarySelection.Tag(tag = renamed))
                }
            }
        }
    }

    fun removeTag(tag: TagNameModel) {
        val tagged = services.library.catalog.value?.documents?.values.orEmpty().filter { tag in it.tags }
        dialogs.ask(
            request = ConfirmationRequestModel(
                title = "Remove tag \"${tag.value}\"",
                message = "The tag is removed from the catalog and from ${tagged.size} document(s). " +
                    "The files stay as they are.",
                affected = tagged.map { it.path.value },
                confirmLabel = "Remove tag",
            ),
        ) {
            edit(edit = CatalogEdit.RemoveTag(tag = tag)) {
                select(LibrarySelection.Folder(path = RelativePathModel.ROOT))
            }
        }
    }

    /**
     * Saves [edit]; reports errors always, and success only when [done] says what happened. [onDone] runs
     * only once the catalog took the edit.
     */
    fun edit(edit: CatalogEdit, done: String? = null, onDone: () -> Unit = {}) {
        session.perform(label = "Saving the catalog") {
            val outcome = services.documents.editCatalog(edit = edit)
            if (outcome !is OperationOutcome.Done || done != null) {
                session.report(message = OutcomeMessages.of(outcome = outcome, done = done.orEmpty()))
            }
            if (outcome is OperationOutcome.Done) {
                onDone()
            }
        }
    }

    /** Asks for a category name and passes it on trimmed, unless it is blank. */
    private fun askCategoryName(title: String, initial: String, confirm: String, submit: (String) -> Unit) {
        askText(title = title, label = "Name", initial = initial, confirm = confirm) { name ->
            val trimmed = name.trim()
            if (trimmed.isEmpty()) {
                session.report(message = UiMessageModel(text = "The name is empty", isError = true))
            } else {
                submit(trimmed)
            }
        }
    }

    private fun askText(title: String, label: String, initial: String, confirm: String, submit: (String) -> Unit) {
        dialogs.askText(
            request = TextInputRequestModel(
                title = title,
                label = label,
                initialValue = initial,
                confirmLabel = confirm,
            ),
            submit = submit,
        )
    }
}
