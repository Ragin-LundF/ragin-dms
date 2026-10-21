package dms.application.port.assistant.model

import dms.application.port.ai.model.ClassificationFailure
import dms.domain.taxonomy.TagName

/** What the AI answered, from [model], or why it did not. */
sealed interface AssistantAnswer {

    data class Text(val text: String, val model: String) : AssistantAnswer

    data class Tags(val tags: List<TagName>, val model: String) : AssistantAnswer

    data class Failed(val reason: ClassificationFailure, val detail: String? = null) : AssistantAnswer
}
