package dms.application.port.ai.model.layout

import dms.application.port.ai.model.ClassificationFailure
import dms.domain.rebuild.model.PageLayoutModel

/** The layout of one page, or why the AI could not read it. */
sealed interface LayoutOutcome {

    data class Read(val layout: PageLayoutModel) : LayoutOutcome

    data class Failure(val reason: ClassificationFailure, val detail: String? = null) : LayoutOutcome
}
