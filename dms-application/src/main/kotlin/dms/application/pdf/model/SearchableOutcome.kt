package dms.application.pdf.model

import dms.application.common.OperationOutcome

/** What "Make searchable" did with one document. */
sealed interface SearchableOutcome {

    /** The new version with a text layer replaced the file; [outcome] says how the write went. */
    data class Made(val outcome: OperationOutcome) : SearchableOutcome

    /** The PDF has a text layer already; nothing changed. */
    data object AlreadySearchable : SearchableOutcome

    /** There is no transcription to add yet; let the AI read the scan first, or use precise OCR. */
    data object NoTranscription : SearchableOutcome

    /** Precise OCR was asked for, but the OCR program is not installed. */
    data object OcrUnavailable : SearchableOutcome

    data object NotPdf : SearchableOutcome

    /** The library is not open, the document is gone, or the PDF could not be read. */
    data class Failed(val outcome: OperationOutcome) : SearchableOutcome
}
