package dms.storage.state.dto

/** `activity.yaml`: recent activity, newest first. */
data class ActivityLogDto(
    val entries: List<ActivityEntryDto> = emptyList(),
)
