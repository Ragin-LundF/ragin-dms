package dms.pdf.ocr

import java.io.IOException
import java.util.concurrent.TimeUnit

/** Runs a program with a time limit; its output is discarded. */
class ProcessRunner(private val timeoutMinutes: Long = DEFAULT_TIMEOUT_MINUTES) : CommandRunner {

    override fun run(command: List<String>): Int? {
        val process = try {
            ProcessBuilder(command)
                .redirectErrorStream(true)
                .redirectOutput(ProcessBuilder.Redirect.DISCARD)
                .start()
        } catch (ignored: IOException) {
            return null
        }
        if (!process.waitFor(timeoutMinutes, TimeUnit.MINUTES)) {
            process.destroyForcibly()
            return null
        }
        return process.exitValue()
    }

    private companion object {
        const val DEFAULT_TIMEOUT_MINUTES = 10L
    }
}
