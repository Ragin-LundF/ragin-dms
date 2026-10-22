package dms.application.document.model

import dms.application.common.OperationOutcome
import dms.application.port.storage.model.ExternalFileModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel

/**
 * What happened to one file the user added. [path] is where it was placed (with a suffix if the
 * name was taken); [duplicates] are documents that already had the same content.
 */
data class AddedFileModel(
    val source: ExternalFileModel,
    val outcome: OperationOutcome,
    val path: RelativePathModel? = null,
    val duplicates: List<DocumentModel> = emptyList(),
)
