package dms.ui.library.drag

import dms.domain.taxonomy.model.CategoryIdModel
import dms.ui.library.drag.model.DragItem
import dms.ui.library.drag.model.DropTarget
import dms.ui.library.model.CategoryEntryModel

/** What dropping on a target does, in words, for the drag preview. */
object DropTexts {

    fun action(item: DragItem, target: DropTarget, categories: List<CategoryEntryModel>): String {
        if (item is DragItem.DraggedCategory) {
            return when (target) {
                is DropTarget.Category -> "Move into ${nameOf(categoryId = target.categoryId, categories = categories)}"
                else -> "Move to the top level"
            }
        }
        return when (target) {
            is DropTarget.Folder -> "Move to ${target.path.value.ifEmpty { "the top level" }}"
            is DropTarget.Category -> "File under ${nameOf(categoryId = target.categoryId, categories = categories)}"

            DropTarget.Uncategorized -> "Remove the category"
            is DropTarget.Tag -> "Tag with ${target.tag.value}"
            DropTarget.TopLevelCategory -> "Drop on a folder, category or tag"
        }
    }

    private fun nameOf(categoryId: CategoryIdModel, categories: List<CategoryEntryModel>): String {
        return categories.firstOrNull { it.category.id == categoryId }?.category?.name ?: "this category"
    }
}
