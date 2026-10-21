package dms.ui.assistant.model

import dms.application.port.assistant.model.ChatTurn
import dms.domain.document.Document

/**
 * The conversation about one document: the [turns] so far, the question being typed in [input],
 * [waiting] while the AI answers, and the [problem] of the last question, which then is back in
 * [input] to send again.
 */
data class ChatState(
    val document: Document,
    val turns: List<ChatTurn> = emptyList(),
    val input: String = "",
    val waiting: Boolean = false,
    val problem: String? = null,
)
