package dms.storage.scan.dto

data class ScanCacheDto(
    val entries: Map<String, ScanCacheEntryDto> = emptyMap(),
)
