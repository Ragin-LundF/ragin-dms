package dms.ui.library.adding

import dms.application.common.OperationOutcome
import dms.application.document.model.AddDocumentsOutcome
import dms.application.port.storage.model.ExternalFileModel
import dms.domain.library.model.RelativePathModel
import dms.ui.PlatformActions
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.UiMessageModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

/**
 * Adding files to the library: chosen in a file dialog or dropped, copied into a folder of every library
 * folder. Without a target they go into the [filteredFolder], the folder the library filters by, or the
 * top level.
 */
class FileAdder(
    private val services: AppServices,
    private val session: LibrarySession,
    private val platform: PlatformActions,
    private val scope: CoroutineScope,
    private val filteredFolder: () -> RelativePathModel,
) {

    /** Asks for files and copies them into the filtered folder, or the top level, of every library folder. */
    fun addFiles() {
        chooseAndAdd(target = filteredFolder())
    }

    /** Asks for files and copies them into the top level of every library folder. */
    fun addFilesToTopLevel() {
        chooseAndAdd(target = RelativePathModel.ROOT)
    }

    /** Asks for files and copies them into [target] of every library folder, as "Add files here" does. */
    fun chooseAndAdd(target: RelativePathModel) {
        scope.launch {
            val files = platform.chooseFiles(title = "Add files to the library")
            if (files.isNotEmpty()) {
                addFiles(locations = files, target = target)
            }
        }
    }

    /** Copies the files at [locations] into [target] of every library folder and says how many were added. */
    fun addFiles(locations: List<String>, target: RelativePathModel = filteredFolder()) {
        if (locations.isEmpty()) {
            session.report(message = UiMessageModel(text = "Only files can be added, not folders", isError = true))
            return
        }
        session.perform(label = "Adding ${locations.size} file(s)") {
            val outcome = services.documents.addDocuments(
                files = locations.map { ExternalFileModel(location = it) },
                targetFolder = target,
            )
            val added = when (outcome) {
                is AddDocumentsOutcome.Added -> outcome.files
                AddDocumentsOutcome.LibraryNotOpen -> {
                    session.report(message = OutcomeMessages.of(outcome = OperationOutcome.LibraryNotOpen, done = ""))
                    return@perform
                }
            }
            val done = added.count { it.outcome is OperationOutcome.Done }
            val duplicates = added.count { it.duplicates.isNotEmpty() }
            val problem = added.firstOrNull { it.outcome !is OperationOutcome.Done }
                ?.let { "${it.source.fileName}: ${OutcomeMessages.of(outcome = it.outcome, done = "").text}" }
            val text = listOfNotNull(
                "Added $done of ${locations.size} file(s)",
                duplicates.takeIf { it > 0 }?.let { "$it already in the library under another name" },
                problem,
            ).joinToString(separator = "; ")
            session.report(message = UiMessageModel(text = text, isError = done < locations.size))
        }
    }
}
