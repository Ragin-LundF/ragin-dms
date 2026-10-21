package dms.storage.state.dto.task

/** `tasks.yaml`: pending suggestions and the keys of dismissed tasks. */
data class TasksDto(
    val suggestions: List<SuggestionDto> = emptyList(),
    val dismissed: List<String> = emptyList(),
)
