package dms.ui.system

/** What the error dialog says about an unexpected failure, instead of the bare exception message. */
object ErrorTexts {

    const val TITLE = "Ragin-DMS stopped working"

    fun messageOf(throwable: Throwable): String {
        // A class that cannot be loaded almost always means the app's files were rebuilt or replaced
        // while it was running; the message of such an error is only the class name.
        if (throwable is LinkageError) {
            return "A part of Ragin-DMS could not be loaded (${throwable.message}). This happens when the app " +
                "is rebuilt or updated while it is running. Restart Ragin-DMS."
        }
        val name = throwable::class.simpleName ?: "Error"
        val detail = throwable.message?.takeIf { it.isNotBlank() }?.let { ": $it" }.orEmpty()
        return "Something unexpected went wrong ($name$detail). The window closes now; open Ragin-DMS again " +
            "to continue."
    }
}
