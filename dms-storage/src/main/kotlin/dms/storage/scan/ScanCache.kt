package dms.storage.scan

import dms.domain.change.model.FileFactsModel
import dms.domain.document.model.ContentHashModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.storage.scan.dto.ScanCacheDto
import dms.storage.scan.dto.ScanCacheEntryDto
import dms.storage.yaml.YamlFile
import tools.jackson.core.JacksonException
import java.nio.file.Path

/**
 * Size, modification time and hash of every file at the last scan of one folder, kept on this device.
 * It lets a scan hash only files that are new or changed, so files a sync client keeps only in the
 * cloud are not downloaded by every scan.
 */
class ScanCache private constructor(
    private val file: YamlFile<ScanCacheDto>,
    private val entries: MutableMap<String, ScanCacheEntryDto>,
) {

    /**
     * The facts of one file. Hashes it when it is unknown to the catalog, differs from the catalog in
     * size, or changed since the last scan; an unchanged catalogued file is trusted.
     */
    fun facts(
        path: RelativePathModel,
        size: Long,
        modified: Long,
        known: DocumentModel?,
        hash: () -> ContentHashModel,
    ): FileFactsModel {
        val cached = entries[path.value]
        val unchanged = cached != null && cached.size == size && cached.modified == modified
        val contentHash = when {
            unchanged -> cached.contentHash?.let { ContentHashModel.fromHex(hex = it) }
            known == null || known.size != size || cached != null -> hash()
            else -> null
        }
        entries[path.value] = ScanCacheEntryDto(size = size, modified = modified, contentHash = contentHash?.value)
        return FileFactsModel(size = size, contentHash = contentHash)
    }

    /** Saves what is known so far, in the middle of a scan. */
    fun checkpoint() {
        file.write(value = ScanCacheDto(entries = entries.toSortedMap()))
    }

    /** Saves the cache, forgetting files that no longer exist. */
    fun save(present: Set<RelativePathModel>) {
        val keep = present.map { it.value }.toSet()
        entries.keys.retainAll(keep)
        file.write(value = ScanCacheDto(entries = entries.toSortedMap()))
    }

    companion object {

        /** The cache in [path]; an unreadable cache starts empty, it only costs hashing time. */
        fun load(path: Path): ScanCache {
            val file = YamlFile(path = path, type = ScanCacheDto::class.java)
            val entries = try {
                file.read()?.entries.orEmpty()
            } catch (ignored: JacksonException) {
                emptyMap()
            }
            return ScanCache(file = file, entries = entries.toMutableMap())
        }
    }
}
