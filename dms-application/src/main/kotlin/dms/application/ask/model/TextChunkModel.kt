package dms.application.ask.model

/** A passage of a document's text: characters [start] to [end] of it. */
data class TextChunkModel(
    val start: Int,
    val end: Int,
    val text: String,
)
