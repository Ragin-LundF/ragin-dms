package dms.ui.library.model

import dms.domain.library.RelativePath
import dms.domain.taxonomy.CategoryId
import dms.domain.taxonomy.TagName

/** What the sidebar has selected; it decides which documents the list shows. */
sealed interface LibrarySelection {

    /** A folder with everything below it; the root shows every document. */
    data class Folder(val path: RelativePath) : LibrarySelection

    data class Category(val categoryId: CategoryId) : LibrarySelection

    data object Uncategorized : LibrarySelection

    data class Tag(val tag: TagName) : LibrarySelection

    /** The documents added lately, newest first. */
    data object Recent : LibrarySelection
}
