package dms.ui.session

import dms.application.common.OperationOutcome
import dms.application.library.model.ConfigureLibraryOutcome
import dms.application.port.storage.model.StorageFailure
import dms.application.sync.model.RetryReport
import dms.application.sync.model.SyncOutcome
import dms.domain.catalog.CatalogRejection
import dms.domain.library.FileNameViolation
import dms.ui.session.model.UiMessage

/** The words for every outcome a use case can return. Use cases return data; only the UI phrases it. */
object OutcomeMessages {

    fun of(outcome: OperationOutcome, done: String): UiMessage {
        return when (outcome) {
            is OperationOutcome.Done -> done(text = done, pendingFolders = outcome.pendingFolders.size)
            OperationOutcome.LibraryNotOpen -> problem(text = "No library is open")
            OperationOutcome.NotFound -> problem(text = "The document or file no longer exists")
            OperationOutcome.ScanOutdated -> problem(text = "The folders changed since the last scan; scan again")
            OperationOutcome.NotConfirmed -> problem(text = "Nothing was changed without confirmation")
            is OperationOutcome.InvalidName -> problem(text = nameProblem(violation = outcome.violation))
            OperationOutcome.NameTaken -> problem(text = "Another entry there already has this name (ignoring case)")
            is OperationOutcome.Rejected -> problem(text = rejection(reason = outcome.reason))
            is OperationOutcome.StorageFailed -> problem(
                text = "The primary folder could not be changed (${failure(reason = outcome.failure.reason)}); " +
                    "nothing was changed",
            )
        }
    }

    fun sync(outcome: SyncOutcome, done: String): UiMessage {
        return when (outcome) {
            SyncOutcome.LibraryNotOpen -> problem(text = "No library is open")
            is SyncOutcome.PrimaryFailed -> problem(
                text = "The primary folder could not be read (${failure(reason = outcome.failure.reason)})",
            )

            is SyncOutcome.Completed -> {
                val report = outcome.report
                val parts = listOfNotNull(
                    "${report.actions.size} file(s) copied or moved",
                    report.conflicts.size.takeIf { it > 0 }?.let { "$it conflict(s) to resolve" },
                    report.failedPaths.size.takeIf { it > 0 }?.let { "$it file(s) failed" },
                    report.unreachableFolders.size.takeIf { it > 0 }?.let { "$it folder(s) offline" },
                )
                UiMessage(text = "$done: ${parts.joinToString(separator = ", ")}")
            }
        }
    }

    fun retry(report: RetryReport?): UiMessage {
        if (report == null) {
            return problem(text = "No library is open")
        }
        val text = "Retried: ${report.succeeded} done, ${report.dropped} no longer needed, " +
            "${report.stillPending} still pending"
        return UiMessage(text = text, isError = report.stillPending > 0)
    }

    /** `null` when the folders were changed; the library is opened again then. */
    fun configure(outcome: ConfigureLibraryOutcome): UiMessage? {
        return when (outcome) {
            is ConfigureLibraryOutcome.Configured -> null
            ConfigureLibraryOutcome.AlreadyConfigured -> problem(text = "A primary folder is already configured")
            ConfigureLibraryOutcome.NotConfigured -> problem(text = "Choose a primary folder first")
            ConfigureLibraryOutcome.LocationAlreadyUsed -> problem(text = "This folder is already part of the library")
            ConfigureLibraryOutcome.UnknownFolder -> problem(text = "This folder is no longer configured")
            ConfigureLibraryOutcome.PrimaryCannotBeRemoved -> problem(
                text = "The primary folder cannot be removed; make another folder the primary first",
            )
        }
    }

    fun failure(reason: StorageFailure): String {
        return when (reason) {
            StorageFailure.FOLDER_UNREACHABLE -> "folder not reachable"
            StorageFailure.ALREADY_EXISTS -> "the target already exists"
            StorageFailure.NOT_FOUND -> "file not found"
            StorageFailure.VERIFICATION_FAILED -> "the copy did not match the original"
            StorageFailure.IO_ERROR -> "read or write error"
            StorageFailure.UNSUPPORTED_FORMAT -> "written by a newer version or damaged"
        }
    }

    fun rejection(reason: CatalogRejection): String {
        return when (reason) {
            CatalogRejection.DOCUMENT_ALREADY_EXISTS -> "The document is already in the catalog"
            CatalogRejection.DOCUMENT_NOT_FOUND -> "The document is no longer in the catalog"
            CatalogRejection.PATH_ALREADY_USED -> "Another document already uses this path"
            CatalogRejection.INVALID_FOLDER_MOVE -> "A folder cannot be moved into itself"
            CatalogRejection.CATEGORY_ALREADY_EXISTS -> "The category already exists"
            CatalogRejection.CATEGORY_NOT_FOUND -> "The category no longer exists"
            CatalogRejection.CATEGORY_NAME_TAKEN -> "A category with this name already exists there"
            CatalogRejection.CATEGORY_IN_USE -> "Documents still use this category"
            CatalogRejection.CATEGORY_HAS_CHILDREN -> "The category still has subcategories"
            CatalogRejection.CATEGORY_CYCLE -> "A category cannot be placed below itself"
            CatalogRejection.TAG_NOT_FOUND -> "The tag no longer exists"
        }
    }

    fun nameProblem(violation: FileNameViolation): String {
        return when (violation) {
            FileNameViolation.EMPTY -> "The name is empty"
            FileNameViolation.TOO_LONG -> "The name is too long"
            FileNameViolation.CONTROL_CHARACTER -> "The name contains control characters"
            FileNameViolation.FORBIDDEN_CHARACTER -> "The name contains one of < > : \" / \\ | ? *"
            FileNameViolation.RESERVED_NAME -> "Windows reserves this name"
            FileNameViolation.LEADING_DOT_OR_SPACE -> "The name must not start with a dot or a space"
            FileNameViolation.TRAILING_DOT_OR_SPACE -> "The name must not end with a dot or a space"
        }
    }

    private fun done(text: String, pendingFolders: Int): UiMessage {
        if (pendingFolders == 0) {
            return UiMessage(text = text)
        }
        return UiMessage(text = "$text; $pendingFolders sync folder(s) get the change when retried")
    }

    private fun problem(text: String): UiMessage {
        return UiMessage(text = text, isError = true)
    }
}
