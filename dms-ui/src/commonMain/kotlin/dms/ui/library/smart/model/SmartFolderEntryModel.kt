package dms.ui.library.smart.model

import dms.domain.query.model.SavedSearchIdModel

/**
 * A smart folder in the sidebar: its name, how many documents it finds ([count] is `null` while it
 * needs the documents' text), and whether it names a category, tag or field that no longer exists.
 */
data class SmartFolderEntryModel(
    val id: SavedSearchIdModel,
    val name: String,
    val count: Int?,
    val invalid: Boolean,
)
