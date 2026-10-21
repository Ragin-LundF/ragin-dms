package dms.ui.library.drag

import dms.ui.library.drag.model.DropTarget
import dms.ui.library.model.CategoryEntry

/** What dropping on a target does, in words, for the drag preview. */
object DropTexts {

    fun action(target: DropTarget, categories: List<CategoryEntry>): String {
        return when (target) {
            is DropTarget.Folder -> "Move to ${target.path.value.ifEmpty { "the top level" }}"
            is DropTarget.Category -> {
                val name = categories.firstOrNull { it.category.id == target.categoryId }?.category?.name
                "File under ${name ?: "this category"}"
            }

            DropTarget.Uncategorized -> "Remove the category"
            is DropTarget.Tag -> "Tag with ${target.tag.value}"
        }
    }
}
