package dms.application.port.ai

import dms.application.port.ai.model.layout.LayoutOutcome
import dms.application.port.ai.model.layout.LineOutcome
import dms.application.port.ai.model.PageImageModel

/**
 * Reads the layout of a scanned page with the vision model the user configured: every printed text line
 * with its box, and the areas that are no text. Cancelling the caller cancels the request.
 */
interface PageLayoutReader {

    suspend fun readLayout(page: PageImageModel): LayoutOutcome

    /** Reads one line again from [line], a crop of the page, as plain text. */
    suspend fun readLine(line: PageImageModel): LineOutcome
}
