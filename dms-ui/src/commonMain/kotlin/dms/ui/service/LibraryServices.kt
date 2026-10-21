package dms.ui.service

import dms.application.library.BuildDashboard
import dms.application.library.ConfigureLibrary
import dms.application.library.DetectDocumentDates
import dms.application.library.OpenLibrary
import dms.application.library.ScanLibrary
import dms.application.port.state.SettingsRepository
import dms.application.port.system.Clock
import dms.application.search.SearchContents
import dms.domain.catalog.Catalog
import kotlinx.coroutines.flow.StateFlow

/** Opening, configuring, scanning and searching the library, and the catalog of the open library. */
class LibraryServices(
    val catalog: StateFlow<Catalog?>,
    val settings: SettingsRepository,
    val clock: Clock,
    val configureLibrary: ConfigureLibrary,
    val openLibrary: OpenLibrary,
    val scanLibrary: ScanLibrary,
    val detectDocumentDates: DetectDocumentDates,
    val buildDashboard: BuildDashboard,
    val searchContents: SearchContents,
)
