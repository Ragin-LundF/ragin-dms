package dms.application.pdf.rebuild.model

import dms.application.common.OperationOutcome
import dms.application.port.pdf.model.RebuiltFileModel

/** What "Convert to real PDF" produced for one document. */
sealed interface RebuildOutcome {

    /** The rebuilt PDF waits for the user's choice; the original is [originalSize] bytes. */
    data class Rebuilt(val rebuilt: RebuiltFileModel, val originalSize: Long) : RebuildOutcome

    /** The document cannot be rebuilt, for [reason]; nothing was written. */
    data class NotAvailable(val reason: RebuildBlocker) : RebuildOutcome

    /** The library is not open, the document is gone, or the PDF could not be read or written. */
    data class Failed(val outcome: OperationOutcome) : RebuildOutcome
}
