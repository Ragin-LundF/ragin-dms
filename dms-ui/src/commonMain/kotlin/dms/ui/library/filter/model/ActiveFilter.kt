package dms.ui.library.filter.model

import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.library.model.YearFilterModel

/** A filter the library applies now, shown as a chip the user can remove. */
sealed interface ActiveFilter {

    data class Tag(val tag: TagNameModel) : ActiveFilter

    data class Category(val categoryId: CategoryIdModel, val name: String) : ActiveFilter

    data class Year(val year: YearFilterModel) : ActiveFilter

    /** The search text; [inContent] while it searches the documents' text as well as their names. */
    data class Text(val query: String, val inContent: Boolean) : ActiveFilter
}
