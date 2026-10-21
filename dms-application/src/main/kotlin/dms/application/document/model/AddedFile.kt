package dms.application.document.model

import dms.application.common.OperationOutcome
import dms.application.port.storage.model.ExternalFile
import dms.domain.document.Document
import dms.domain.library.RelativePath

/**
 * What happened to one file the user added. [path] is where it was placed (with a suffix if the
 * name was taken); [duplicates] are documents that already had the same content.
 */
data class AddedFile(
    val source: ExternalFile,
    val outcome: OperationOutcome,
    val path: RelativePath? = null,
    val duplicates: List<Document> = emptyList(),
)
