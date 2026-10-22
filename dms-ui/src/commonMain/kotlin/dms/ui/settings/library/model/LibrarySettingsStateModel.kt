package dms.ui.settings.library.model

import dms.domain.settings.model.CatalogSettingsModel

/** The library-wide settings as the catalog has them now; `null` while no library is open. */
data class LibrarySettingsStateModel(
    val settings: CatalogSettingsModel? = null,
    val busy: String? = null,
)
