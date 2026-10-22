package dms.domain.settings.model

import dms.domain.field.model.CurrencyCodeModel

/**
 * How the library is organised: the [inbox], the path template the restructure assistant saved
 * last (inbox filing uses it as a pre-fill), the currency money fields use when none is given, and
 * whether approved metadata is also written into PDF files ([embedMetadata], off by default).
 */
data class LibraryLayoutModel(
    val inbox: InboxSettingsModel = InboxSettingsModel(),
    val pathTemplate: String? = null,
    val defaultCurrency: CurrencyCodeModel? = null,
    val embedMetadata: Boolean = false,
) {
    init {
        require(value = pathTemplate == null || pathTemplate.isNotBlank()) { "Path template must be null or not blank" }
    }
}
