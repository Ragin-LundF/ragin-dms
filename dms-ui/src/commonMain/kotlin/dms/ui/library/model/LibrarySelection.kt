package dms.ui.library.model

import dms.domain.library.model.RelativePathModel
import dms.domain.query.model.SavedSearchIdModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.TagNameModel

/** What the sidebar has selected; it decides which documents the list shows. */
sealed interface LibrarySelection {

    /** A folder with everything below it; the root shows every document. */
    data class Folder(val path: RelativePathModel) : LibrarySelection

    data class Category(val categoryId: CategoryIdModel) : LibrarySelection

    data object Uncategorized : LibrarySelection

    data class Tag(val tag: TagNameModel) : LibrarySelection

    /** The documents added lately, newest first. */
    data object Recent : LibrarySelection

    /** The documents of a saved search. */
    data class SmartFolder(val id: SavedSearchIdModel) : LibrarySelection
}
