package dms.application.port.assistant.model

/** One passage the answer may draw on, numbered as it is cited: `[number]`. */
data class SourceChunkModel(
    val number: Int,
    val title: String,
    val text: String,
)
