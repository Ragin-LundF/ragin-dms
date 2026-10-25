package dms.ui.document.model

import dms.domain.taxonomy.model.TagNameModel

/**
 * The key facts of a document for the quick look; `null` where it has no value. [amount] and [due]
 * are `null` while the library defines no amount or due-date field.
 */
data class QuickFactsModel(
    val category: String?,
    val tags: List<TagNameModel>,
    val correspondent: String?,
    val documentDate: String?,
    val amount: DocumentFactModel?,
    val due: DocumentFactModel?,
)
