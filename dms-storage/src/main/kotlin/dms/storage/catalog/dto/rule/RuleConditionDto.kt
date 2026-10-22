package dms.storage.catalog.dto.rule

import com.fasterxml.jackson.annotation.JsonInclude
import com.fasterxml.jackson.annotation.JsonPropertyOrder

/**
 * One rule condition. [type] is `correspondent` (with [name]), `textAny` or `textAll` (with [words])
 * or `fileName` (with [glob]).
 */
@JsonInclude(JsonInclude.Include.NON_EMPTY)
@JsonPropertyOrder("type", "name", "words", "glob")
data class RuleConditionDto(
    val type: String,
    val name: String? = null,
    val words: List<String> = emptyList(),
    val glob: String? = null,
)
