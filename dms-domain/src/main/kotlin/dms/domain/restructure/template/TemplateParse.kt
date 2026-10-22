package dms.domain.restructure.template

import dms.domain.restructure.template.model.PathTemplateModel

/** The result of reading a path template. */
sealed interface TemplateParse {

    data class Parsed(val template: PathTemplateModel) : TemplateParse

    data class Invalid(val problem: TemplateProblem) : TemplateParse
}
