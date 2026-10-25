package dms.ui.library.filter.model

import dms.domain.document.DocumentKind
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.library.model.YearFilterModel

/** One active facet filter; the search field shows it as a token and its facet chip as its value. */
sealed interface FacetFilter {

    /** The category with its subcategories; `null` for the documents without a category. */
    data class Category(val categoryId: CategoryIdModel?) : FacetFilter

    data class Tag(val tag: TagNameModel) : FacetFilter

    data class Year(val year: YearFilterModel) : FacetFilter

    /** The documents from this correspondent, ignoring case. */
    data class Correspondent(val name: String) : FacetFilter

    data class Type(val kind: DocumentKind) : FacetFilter

    /** The folder with everything below it. */
    data class Folder(val path: RelativePathModel) : FacetFilter
}
