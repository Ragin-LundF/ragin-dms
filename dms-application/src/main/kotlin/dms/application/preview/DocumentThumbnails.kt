package dms.application.preview

import dms.application.library.CatalogStore
import dms.application.port.ai.DocumentPreviewer
import dms.application.port.ai.model.PageImageModel
import dms.application.preview.model.ThumbnailOutcome
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/**
 * Thumbnails of the files in the library folders, also of files the catalog does not know yet. The
 * most recent ones are kept in memory, so a list drawn again does not render its files again; a file
 * without a picture is remembered as such. Renders one file at a time and never writes.
 */
class DocumentThumbnails(
    private val store: CatalogStore,
    private val previewer: DocumentPreviewer,
) {

    private val mutex = Mutex()
    private val cache = object : LinkedHashMap<Pair<LibraryFolderIdModel, RelativePathModel>, PageImageModel?>(
        CAPACITY,
        LOAD_FACTOR,
        true,
    ) {
        override fun removeEldestEntry(
            eldest: MutableMap.MutableEntry<Pair<LibraryFolderIdModel, RelativePathModel>, PageImageModel?>?,
        ): Boolean {
            return size > CAPACITY
        }
    }

    suspend operator fun invoke(folderId: LibraryFolderIdModel, path: RelativePathModel): ThumbnailOutcome {
        val folder = store.current()?.folders?.find(id = folderId) ?: return ThumbnailOutcome.FolderNotFound
        val key = folderId to path
        val image = mutex.withLock {
            if (cache.containsKey(key)) {
                cache[key]
            } else {
                previewer.thumbnail(folder = folder, path = path, maxSide = MAX_SIDE).also { cache[key] = it }
            }
        }
        return image?.let { ThumbnailOutcome.Rendered(image = it) } ?: ThumbnailOutcome.NoPicture
    }

    private companion object {
        const val CAPACITY = 200
        const val LOAD_FACTOR = 0.75f

        /** Sharp enough for a small picture on a high-resolution screen. */
        const val MAX_SIDE = 320
    }
}
