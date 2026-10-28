package dms.application.port.ai.model.layout

import dms.application.port.ai.model.ClassificationFailure

/** The text of one line read again, or why the AI could not read it. */
sealed interface LineOutcome {

    data class Read(val text: String) : LineOutcome

    data class Failure(val reason: ClassificationFailure, val detail: String? = null) : LineOutcome
}
