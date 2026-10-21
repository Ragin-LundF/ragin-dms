package dms.application.common

import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.library.FileNamePolicy
import dms.domain.library.RelativePath

/**
 * The names a new entry in a folder must not clash with: what the primary folder holds there, plus
 * what the catalog places there, because a sync folder may still hold documents the primary lacks.
 */
class SiblingNames(private val storage: DocumentStorage) {

    /** The names directly inside [parent], or the failure that kept the primary from listing them. */
    suspend fun of(library: LibrarySnapshot, parent: RelativePath): StorageResult<Set<String>> {
        val listed = storage.listNames(folder = library.folders.primary, path = parent)
        if (listed is StorageResult.Failure && listed.reason != StorageFailure.NOT_FOUND) {
            return listed
        }
        val onDisk = (listed as? StorageResult.Success)?.value.orEmpty()
        val depth = parent.segments.size
        val catalogued = library.catalog.documentsInside(folder = parent).map { it.path.segments[depth] }
        return StorageResult.Success(value = (onDisk + catalogued).toSet())
    }

    /**
     * `true` if nothing else in its folder has the name of [target]. [moving] is the entry being moved
     * or renamed there, which does not clash with itself, so a rename that only changes the case works.
     */
    suspend fun isFree(library: LibrarySnapshot, target: RelativePath, moving: RelativePath?): StorageResult<Boolean> {
        val parent = target.parent ?: RelativePath.ROOT
        val siblings = of(library = library, parent = parent)
        if (siblings is StorageResult.Failure) {
            return siblings
        }
        val others = (siblings as StorageResult.Success).value.filterNot { name ->
            moving != null &&
                moving.parent == parent &&
                FileNamePolicy.isSameName(first = name, second = moving.fileName)
        }
        return StorageResult.Success(value = !FileNamePolicy.collidesWithAny(name = target.fileName, siblings = others))
    }
}
