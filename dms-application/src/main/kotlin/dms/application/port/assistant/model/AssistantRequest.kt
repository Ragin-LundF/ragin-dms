package dms.application.port.assistant.model

import dms.application.port.ai.model.DocumentContent

/** The document the AI reads: its current file name and its content as text or page images. */
data class AssistantRequest(
    val fileName: String,
    val content: DocumentContent,
)
