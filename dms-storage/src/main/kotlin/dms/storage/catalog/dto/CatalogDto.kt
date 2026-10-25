package dms.storage.catalog.dto

import com.fasterxml.jackson.annotation.JsonAnyGetter
import com.fasterxml.jackson.annotation.JsonAnySetter
import com.fasterxml.jackson.annotation.JsonInclude
import com.fasterxml.jackson.annotation.JsonPropertyOrder
import dms.storage.catalog.dto.rule.RuleDto
import dms.storage.catalog.dto.settings.CorrespondentDto
import dms.storage.catalog.dto.settings.FieldDefinitionDto
import dms.storage.catalog.dto.settings.LayoutDto
import dms.storage.catalog.dto.settings.SavedSearchDto

/**
 * The catalog file `dms-catalog.yaml`. Fields this version does not know are kept in [unknownFields].
 * Everything after [documents] was added in 1.1 as optional fields, so older versions keep it as
 * unknown fields and older files still load.
 */
@JsonInclude(JsonInclude.Include.NON_EMPTY)
@JsonPropertyOrder(
    "formatVersion",
    "libraryId",
    "revision",
    "layout",
    "fieldDefinitions",
    "savedSearches",
    "correspondents",
    "rules",
    "categories",
    "tags",
    "documents",
    "links",
)
data class CatalogDto(
    val formatVersion: Int,
    val libraryId: String,
    val revision: Long,
    val categories: List<CategoryDto> = emptyList(),
    val tags: List<String> = emptyList(),
    val documents: List<DocumentDto> = emptyList(),
    val links: List<LinkDto> = emptyList(),
    val fieldDefinitions: List<FieldDefinitionDto> = emptyList(),
    val savedSearches: List<SavedSearchDto> = emptyList(),
    val rules: List<RuleDto> = emptyList(),
    val correspondents: List<CorrespondentDto> = emptyList(),
    val layout: LayoutDto? = null,
) : ExtensibleDto {
    @get:JsonAnyGetter
    @field:JsonAnySetter
    override val unknownFields: MutableMap<String, Any?> = linkedMapOf()
}
