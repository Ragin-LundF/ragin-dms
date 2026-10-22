package dms.application.catalog

import dms.application.common.ActivityRecorder
import dms.application.common.OperationOutcome
import dms.application.library.CatalogStore
import dms.application.port.state.model.ActivityKind
import dms.domain.catalog.CatalogUpdate
import dms.domain.settings.model.CatalogSettingsModel

/**
 * Changes the library-wide settings in the catalog, such as the field definitions, saved searches,
 * rules, the default currency or the saved path template, and saves them to every library folder.
 * A change that leaves the settings as they are writes nothing.
 */
class ChangeSettings(
    private val store: CatalogStore,
    private val activity: ActivityRecorder,
) {

    suspend operator fun invoke(
        subject: String,
        change: (CatalogSettingsModel) -> CatalogSettingsModel,
    ): OperationOutcome {
        val outcome = store.commit { catalog ->
            val changed = change(catalog.settings)
            if (changed == catalog.settings) {
                CatalogUpdate.Applied(catalog = catalog)
            } else {
                catalog.changeSettings(settings = changed)
            }
        }
        if (outcome is OperationOutcome.Done) {
            activity.record(kind = ActivityKind.CATALOG_EDITED, subject = subject)
        }
        return outcome
    }
}
