package dms.application.restructure.model

import dms.domain.restructure.model.RestructurePlanModel
import dms.domain.restructure.template.TemplateProblem

sealed interface PlanOutcome {

    data class Planned(val plan: RestructurePlanModel) : PlanOutcome

    data class InvalidTemplate(val problem: TemplateProblem) : PlanOutcome

    data object LibraryNotOpen : PlanOutcome
}
