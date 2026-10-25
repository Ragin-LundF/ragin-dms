package dms.storage.catalog.dto.settings

import com.fasterxml.jackson.annotation.JsonAnyGetter
import com.fasterxml.jackson.annotation.JsonAnySetter
import com.fasterxml.jackson.annotation.JsonPropertyOrder
import dms.storage.catalog.dto.ExtensibleDto

/** The inbox settings; every flag is written, so the file says what is switched on. */
@JsonPropertyOrder("enabled", "path", "aiAnalysis", "aiTargetFolder", "autoAnalyze")
data class InboxDto(
    val enabled: Boolean = false,
    val path: String = "Inbox",
    val aiAnalysis: Boolean = true,
    val aiTargetFolder: Boolean = true,
    val autoAnalyze: Boolean = false,
) : ExtensibleDto {
    @get:JsonAnyGetter
    @field:JsonAnySetter
    override val unknownFields: MutableMap<String, Any?> = linkedMapOf()
}
