package dms.application.export

import dms.application.common.ActivityRecorder
import dms.application.export.model.ExportOptionsModel
import dms.application.export.model.ExportOutcome
import dms.application.export.model.ExportScope
import dms.application.library.CatalogStore
import dms.application.port.export.ExportWriter
import dms.application.port.export.model.ExportTargetModel
import dms.application.port.state.model.ActivityKind
import dms.application.port.storage.model.StorageResult
import dms.application.port.system.Clock
import dms.application.search.SearchContents

/**
 * Exports documents into a folder or a ZIP file outside the library, with an index, for example for
 * the tax adviser. It only reads the library; it never writes into a library folder.
 */
class ExportDocuments(
    private val store: CatalogStore,
    private val search: SearchContents,
    private val writer: ExportWriter,
    private val activity: ActivityRecorder,
    private val clock: Clock,
) {

    suspend operator fun invoke(
        scope: ExportScope,
        directory: String,
        name: String,
        options: ExportOptionsModel,
    ): ExportOutcome {
        val library = store.current() ?: return ExportOutcome.LibraryNotOpen
        if (writer.isInside(directory = directory, folders = library.folders.folders)) {
            return ExportOutcome.InsideLibrary
        }
        val documents = when (scope) {
            is ExportScope.Documents -> scope.ids.mapNotNull { library.catalog.documents[it] }
            is ExportScope.Query -> search.search(query = scope.query)
        }
        if (documents.isEmpty()) {
            return ExportOutcome.Empty
        }
        val bundle = ExportPlanner.plan(
            catalog = library.catalog,
            documents = documents,
            options = options,
            zone = clock.zone(),
        )
        val target = ExportTargetModel(directory = directory, name = name, zip = options.zip)
        return when (val written = writer.write(source = library.folders.primary, bundle = bundle, target = target)) {
            is StorageResult.Failure -> ExportOutcome.Failed(failure = written)
            is StorageResult.Success -> {
                activity.record(
                    kind = ActivityKind.EXPORTED,
                    subject = "${documents.size} documents to ${written.value}",
                )
                ExportOutcome.Exported(count = documents.size, location = written.value)
            }
        }
    }
}
