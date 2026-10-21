package dms.storage.catalog.dto

import com.fasterxml.jackson.annotation.JsonAnyGetter
import com.fasterxml.jackson.annotation.JsonAnySetter
import com.fasterxml.jackson.annotation.JsonInclude
import com.fasterxml.jackson.annotation.JsonPropertyOrder

/** The catalog file `dms-catalog.yaml`. Fields this version does not know are kept in [unknownFields]. */
@JsonInclude(JsonInclude.Include.NON_EMPTY)
@JsonPropertyOrder("formatVersion", "libraryId", "revision", "categories", "tags", "documents")
data class CatalogDto(
    val formatVersion: Int,
    val libraryId: String,
    val revision: Long,
    val categories: List<CategoryDto> = emptyList(),
    val tags: List<String> = emptyList(),
    val documents: List<DocumentDto> = emptyList(),
) {
    @get:JsonAnyGetter
    @field:JsonAnySetter
    val unknownFields: MutableMap<String, Any?> = linkedMapOf()
}
