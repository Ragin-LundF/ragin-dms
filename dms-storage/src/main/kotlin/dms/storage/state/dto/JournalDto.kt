package dms.storage.state.dto

/** `journal.yaml`: operations waiting for a sync folder, oldest first. */
data class JournalDto(
    val operations: List<PendingOperationDto> = emptyList(),
)
