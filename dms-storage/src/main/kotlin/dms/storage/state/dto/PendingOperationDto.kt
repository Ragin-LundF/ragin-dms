package dms.storage.state.dto

import com.fasterxml.jackson.annotation.JsonInclude

/** One journal entry. [type] names the operation; only the fields it needs are set. */
@JsonInclude(JsonInclude.Include.NON_NULL)
data class PendingOperationDto(
    val id: String,
    val folderId: String,
    val recordedAt: String,
    val type: String,
    val path: String? = null,
    val from: String? = null,
    val to: String? = null,
    val replaceExisting: Boolean? = null,
    val documentId: String? = null,
)
