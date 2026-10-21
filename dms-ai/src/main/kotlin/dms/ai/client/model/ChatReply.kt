package dms.ai.client.model

import dms.application.port.ai.model.ClassificationFailure

/** The text the model answered with, from [model], or why there is none. */
sealed interface ChatReply {

    data class Answered(val content: String, val model: String) : ChatReply

    data class Failed(val reason: ClassificationFailure, val detail: String? = null) : ChatReply
}
