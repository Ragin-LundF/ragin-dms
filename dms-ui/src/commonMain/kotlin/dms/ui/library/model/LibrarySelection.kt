package dms.ui.library.model

import dms.domain.library.model.RelativePathModel
import dms.domain.query.model.SavedSearchIdModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.TagNameModel

/**
 * What the library opens. The sidebar's views are [Folder] at the root ("All documents"), [Inbox],
 * [NeedsReview], [Recent], [Folders] and [SmartFolder]. A [Category], [Uncategorized], [Tag] or a folder
 * below the root, as the dashboard and the palette open them, become filters of "All documents".
 */
sealed interface LibrarySelection {

    /** A folder with everything below it; the root shows every document. */
    data class Folder(val path: RelativePathModel) : LibrarySelection

    data class Category(val categoryId: CategoryIdModel) : LibrarySelection

    data object Uncategorized : LibrarySelection

    data class Tag(val tag: TagNameModel) : LibrarySelection

    /** The documents in the inbox folder, while the inbox is switched on. */
    data object Inbox : LibrarySelection

    /** The documents with an AI suggestion that waits for the user's review. */
    data object NeedsReview : LibrarySelection

    /** The documents added in this calendar month, newest first. */
    data object Recent : LibrarySelection

    /** The folder view, a file manager of the library's folders. */
    data object Folders : LibrarySelection

    /** The documents of a saved search, a "saved view" in the sidebar. */
    data class SmartFolder(val id: SavedSearchIdModel) : LibrarySelection
}
