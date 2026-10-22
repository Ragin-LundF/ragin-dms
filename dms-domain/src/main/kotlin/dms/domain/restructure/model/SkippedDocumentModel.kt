package dms.domain.restructure.model

import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.RelativePathModel
import dms.domain.restructure.SkipReason

data class SkippedDocumentModel(
    val documentId: DocumentIdModel,
    val path: RelativePathModel,
    val reason: SkipReason,
)
