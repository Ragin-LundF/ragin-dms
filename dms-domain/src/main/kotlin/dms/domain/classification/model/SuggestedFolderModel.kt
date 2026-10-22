package dms.domain.classification.model

import dms.domain.library.model.RelativePathModel

/** Where the AI suggests filing a document: an existing folder, or a new one ([isNew]) below an existing one. */
data class SuggestedFolderModel(
    val path: RelativePathModel,
    val isNew: Boolean,
) {
    init {
        require(value = !path.isRoot) { "A suggested folder needs a path" }
    }
}
