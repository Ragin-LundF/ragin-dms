package dms.ai.support

import kotlin.time.Duration
import kotlin.time.Duration.Companion.minutes

/** What tests that read or render PDFs share. */
object PdfTests {

    /**
     * The time a PDF test may take. The first font lookup of PDFBox scans every system font to build
     * its font cache, which takes over a minute on a fresh Windows runner, longer than `runTest`'s default.
     */
    val TIMEOUT: Duration = 5.minutes
}
