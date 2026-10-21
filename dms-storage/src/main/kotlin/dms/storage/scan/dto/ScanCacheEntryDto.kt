package dms.storage.scan.dto

import com.fasterxml.jackson.annotation.JsonInclude

/** What the last scan saw of one file; [contentHash] is absent when the file was trusted without hashing. */
@JsonInclude(JsonInclude.Include.NON_NULL)
data class ScanCacheEntryDto(
    val size: Long,
    val modified: Long,
    val contentHash: String? = null,
)
