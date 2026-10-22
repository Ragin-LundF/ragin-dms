package dms.storage.catalog.dto.settings

import com.fasterxml.jackson.annotation.JsonInclude
import com.fasterxml.jackson.annotation.JsonPropertyOrder

@JsonInclude(JsonInclude.Include.NON_NULL)
@JsonPropertyOrder("inbox", "pathTemplate", "defaultCurrency", "embedMetadata")
data class LayoutDto(
    val inbox: InboxDto? = null,
    val pathTemplate: String? = null,
    val defaultCurrency: String? = null,
    val embedMetadata: Boolean? = null,
)
