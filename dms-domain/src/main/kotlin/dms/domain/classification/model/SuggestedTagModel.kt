package dms.domain.classification.model

import dms.domain.taxonomy.model.TagNameModel

/** A suggested tag; [isNew] marks tags the catalog does not know yet, so the UI can highlight them. */
data class SuggestedTagModel(
    val name: TagNameModel,
    val isNew: Boolean,
)
