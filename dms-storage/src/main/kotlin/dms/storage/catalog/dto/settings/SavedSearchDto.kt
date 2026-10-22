package dms.storage.catalog.dto.settings

import com.fasterxml.jackson.annotation.JsonPropertyOrder

@JsonPropertyOrder("id", "name", "query")
data class SavedSearchDto(
    val id: String,
    val name: String,
    val query: QueryDto = QueryDto(),
)
