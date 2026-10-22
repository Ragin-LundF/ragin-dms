package dms.storage.catalog.dto.settings

import com.fasterxml.jackson.annotation.JsonInclude
import com.fasterxml.jackson.annotation.JsonPropertyOrder

@JsonInclude(JsonInclude.Include.NON_EMPTY)
@JsonPropertyOrder("name", "aliases")
data class CorrespondentDto(
    val name: String,
    val aliases: List<String> = emptyList(),
)
