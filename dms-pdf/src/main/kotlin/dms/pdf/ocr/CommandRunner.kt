package dms.pdf.ocr

/** Runs an external program; tests replace it, so they never need the program installed. */
fun interface CommandRunner {

    /** The exit code of [command], or `null` if it could not be started or did not finish in time. */
    fun run(command: List<String>): Int?
}
