package dms.application.restructure

import dms.application.library.CatalogStore
import dms.application.restructure.model.PlanOutcome
import dms.application.restructure.model.RestructureScope
import dms.application.search.ContentMatcher
import dms.domain.restructure.FileNaming
import dms.domain.restructure.RestructurePlanner
import dms.domain.restructure.template.TemplateParse
import dms.domain.restructure.template.model.PathTemplateModel

/** Plans a restructure of a scope by a path template, for the user to review. Changes nothing. */
class PlanRestructure(
    private val store: CatalogStore,
    private val search: ContentMatcher,
) {

    suspend operator fun invoke(scope: RestructureScope, template: String, naming: FileNaming): PlanOutcome {
        val parsed = when (val parse = PathTemplateModel.parse(raw = template)) {
            is TemplateParse.Invalid -> return PlanOutcome.InvalidTemplate(problem = parse.problem)
            is TemplateParse.Parsed -> parse.template
        }
        val library = store.current() ?: return PlanOutcome.LibraryNotOpen
        val catalog = library.catalog
        val documents = when (scope) {
            is RestructureScope.Folder -> catalog.documents.values.filter { it.path.isInside(folder = scope.path) }
            is RestructureScope.Query -> search.matching(library = library, query = scope.query)
        }
        val plan = RestructurePlanner.plan(
            catalog = catalog,
            scope = documents,
            template = parsed,
            naming = naming,
            inbox = catalog.settings.layout.inbox,
        )
        return PlanOutcome.Planned(plan = plan)
    }
}
