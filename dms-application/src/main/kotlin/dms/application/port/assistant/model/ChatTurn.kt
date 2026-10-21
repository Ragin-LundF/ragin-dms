package dms.application.port.assistant.model

/** One message of a conversation about a document. */
data class ChatTurn(
    val role: ChatRole,
    val text: String,
)
