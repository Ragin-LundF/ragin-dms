package dms.ui.pdf.rebuild

import dms.application.pdf.rebuild.model.RebuildBlocker
import dms.application.pdf.rebuild.model.RebuildOutcome
import dms.domain.rebuild.KeepReason
import dms.domain.rebuild.model.RebuiltPageModel
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.UiMessageModel

/** The words for converting a scan to a real PDF: outcomes and why lines stayed images. */
object RebuildTexts {

    fun messageOf(outcome: RebuildOutcome, name: String): UiMessageModel {
        return when (outcome) {
            is RebuildOutcome.Rebuilt -> UiMessageModel(
                text = failedPages(pages = outcome.rebuilt.pages) ?: "$name is rebuilt: compare and choose",
            )
            is RebuildOutcome.NotAvailable -> UiMessageModel(
                text = blocker(reason = outcome.reason, name = name),
                isError = true,
            )
            is RebuildOutcome.Failed -> OutcomeMessages.of(outcome = outcome.outcome, done = "")
        }
    }

    fun reason(reason: KeepReason): String {
        return when (reason) {
            KeepReason.NOT_SURE -> "the AI was not sure of every character"
            KeepReason.NO_INK -> "no print where the AI saw the line"
            KeepReason.COVERED -> "covered by a stamp, signature or handwriting"
            KeepReason.NOT_ENCODABLE -> "a character the font does not have"
            KeepReason.TEXT_MISMATCH -> "the text does not match the print"
            KeepReason.LOST_CHARACTERS -> "the AI lost characters"
            KeepReason.PAGE_FAILED -> "the AI could not read the page"
        }
    }

    /** "Page 3 stayed an image (the AI could not read it)", or `null` if every page was read. */
    private fun failedPages(pages: List<RebuiltPageModel>): String? {
        val failed = pages.indices.filter { KeepReason.PAGE_FAILED in pages[it].keptReasons }.map { it + 1 }
        if (failed.isEmpty()) return null
        val which = if (failed.size == 1) "Page ${failed.single()}" else "Pages ${failed.joinToString()}"
        val verb = if (failed.size == 1) "stayed an image" else "stayed images"
        return "$which $verb (the AI could not read it in time or at all)"
    }

    private fun blocker(reason: RebuildBlocker, name: String): String {
        return when (reason) {
            RebuildBlocker.NOT_PDF -> "Only PDF files can be converted"
            RebuildBlocker.HAS_TEXT_LAYER -> "$name has real text already"
            RebuildBlocker.AI_NOT_REACHABLE -> "AI is not configured or cannot be reached: check Settings › AI"
            RebuildBlocker.MODEL_CANNOT_READ_IMAGES ->
                "Your AI model cannot read images: choose a vision model in Settings › AI"
            RebuildBlocker.SOURCE_CHANGED -> "$name changed while it was converted: convert it again"
        }
    }
}
