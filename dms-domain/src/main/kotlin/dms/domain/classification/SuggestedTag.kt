package dms.domain.classification

import dms.domain.taxonomy.TagName

/** A suggested tag; [isNew] marks tags the catalog does not know yet, so the UI can highlight them. */
data class SuggestedTag(
    val name: TagName,
    val isNew: Boolean,
)
