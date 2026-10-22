package dms.domain.settings.model

import dms.domain.library.FileNamePolicy
import dms.domain.library.FileNameViolation
import dms.domain.library.model.RelativePathModel

/**
 * The optional inbox: a folder inside the library, at the same relative [path] in every library
 * folder, where new documents are dropped and then filed. [aiAnalysis] lets the AI suggest metadata
 * for inbox documents, [aiTargetFolder] lets it also suggest where to file them, and [autoAnalyze]
 * runs the analysis as soon as a file arrives.
 */
data class InboxSettingsModel(
    val enabled: Boolean = false,
    val path: RelativePathModel = DEFAULT_PATH,
    val aiAnalysis: Boolean = true,
    val aiTargetFolder: Boolean = true,
    val autoAnalyze: Boolean = false,
) {
    init {
        require(value = pathViolation(path = path) == null) { "Invalid inbox path: $path" }
    }

    /** `true` if [candidate] lies in the inbox, while it is switched on. */
    fun contains(candidate: RelativePathModel): Boolean {
        return enabled && candidate.isInside(folder = path)
    }

    companion object {
        val DEFAULT_PATH: RelativePathModel = RelativePathModel.of(raw = "Inbox")

        /**
         * Why [path] cannot be the inbox, or `null` if it can: not the library root itself, and every
         * folder name valid by [FileNamePolicy], which also keeps it out of the hidden `.dms-*` folders.
         */
        fun pathViolation(path: RelativePathModel): FileNameViolation? {
            if (path.isRoot) {
                return FileNameViolation.EMPTY
            }
            return path.segments.firstNotNullOfOrNull { FileNamePolicy.check(name = it) }
        }
    }
}
