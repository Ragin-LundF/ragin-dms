package dms.ui.session

import dms.domain.library.FileNamePolicy
import dms.domain.library.RelativePath

/** Turns a name the user typed into a path, using the one file name policy of the domain. */
object NameInput {

    /** Why [raw] cannot be a file or folder name, or `null` if it can. */
    fun problem(raw: String): String? {
        return FileNamePolicy.check(name = raw.trim())?.let { OutcomeMessages.nameProblem(violation = it) }
    }

    /** [raw] as an entry of [parent]; only valid after [problem] returned `null`. */
    fun child(parent: RelativePath, raw: String): RelativePath {
        return parent.resolve(name = raw.trim())
    }
}
