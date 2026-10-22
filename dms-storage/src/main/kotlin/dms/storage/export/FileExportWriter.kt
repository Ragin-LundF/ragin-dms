package dms.storage.export

import dms.application.port.export.ExportWriter
import dms.application.port.export.model.ExportBundleModel
import dms.application.port.export.model.ExportTargetModel
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.library.FileNamePolicy
import dms.domain.library.model.LibraryFolderModel
import dms.storage.fs.AtomicFiles
import dms.storage.fs.ContentHasher
import dms.storage.fs.PathResolver
import dms.storage.fs.StorageIo
import java.nio.file.Files
import java.nio.file.Path
import java.util.zip.ZipEntry
import java.util.zip.ZipFile
import java.util.zip.ZipOutputStream

/**
 * Writes exports outside the library: a new folder with hash-verified copies, or one ZIP file whose
 * entries are read back and compared with the originals. A folder or file that already exists is never
 * replaced; the export takes the next free name. A failed export leaves no half-written ZIP behind.
 */
class FileExportWriter : ExportWriter {

    override fun isInside(directory: String, folders: List<LibraryFolderModel>): Boolean {
        val target = runCatching { Path.of(directory).toAbsolutePath().normalize() }.getOrNull() ?: return true
        return folders.any { folder ->
            val root = runCatching { Path.of(folder.location.value).toAbsolutePath().normalize() }.getOrNull()
            root != null && (target.startsWith(root) || sameFile(first = target, second = root))
        }
    }

    override suspend fun write(
        source: LibraryFolderModel,
        bundle: ExportBundleModel,
        target: ExportTargetModel,
    ): StorageResult<String> {
        return StorageIo.run {
            val root = PathResolver.root(folder = source)
                ?: return@run StorageIo.failure(reason = StorageFailure.FOLDER_UNREACHABLE)
            val directory = Path.of(target.directory)
            if (!Files.isDirectory(directory)) {
                return@run StorageIo.failure(reason = StorageFailure.NOT_FOUND, detail = target.directory)
            }
            val name = FileNamePolicy.sanitize(name = target.name)
            if (target.zip) {
                writeZip(root = root, bundle = bundle, file = free(directory = directory, name = "$name.zip"))
            } else {
                writeFolder(root = root, bundle = bundle, folder = free(directory = directory, name = name))
            }
        }
    }

    override suspend fun writeText(directory: String, name: String, text: String): StorageResult<String> {
        return StorageIo.run {
            val parent = Path.of(directory)
            if (!Files.isDirectory(parent)) {
                return@run StorageIo.failure(reason = StorageFailure.NOT_FOUND, detail = directory)
            }
            val file = free(directory = parent, name = FileNamePolicy.sanitize(name = name))
            AtomicFiles.write(target = file, bytes = text.toByteArray(Charsets.UTF_8))
            StorageResult.Success(value = file.toString())
        }
    }

    private fun writeFolder(root: Path, bundle: ExportBundleModel, folder: Path): StorageResult<String> {
        Files.createDirectories(folder)
        bundle.entries.forEach { entry ->
            val file = PathResolver.existing(root = root, path = entry.source)
                ?: return StorageIo.failure(reason = StorageFailure.NOT_FOUND, detail = entry.source.value)
            val copied = AtomicFiles.copy(source = file, target = folder.resolve(entry.target))
            if (copied is StorageResult.Failure) {
                return copied
            }
        }
        bundle.index?.let { rows ->
            AtomicFiles.write(target = folder.resolve(CSV), bytes = IndexFiles.csv(bundle = bundle, rows = rows))
            AtomicFiles.write(target = folder.resolve(HTML), bytes = IndexFiles.html(bundle = bundle, rows = rows))
        }
        return StorageResult.Success(value = folder.toString())
    }

    private fun writeZip(root: Path, bundle: ExportBundleModel, file: Path): StorageResult<String> {
        val temp = AtomicFiles.tempFileFor(target = file)
        try {
            val sources = bundle.entries.associate { entry ->
                val source = PathResolver.existing(root = root, path = entry.source)
                    ?: return StorageIo.failure(reason = StorageFailure.NOT_FOUND, detail = entry.source.value)
                entry.target to source
            }
            ZipOutputStream(Files.newOutputStream(temp)).use { zip ->
                sources.forEach { (name, source) ->
                    zip.putNextEntry(ZipEntry(name))
                    Files.copy(source, zip)
                    zip.closeEntry()
                }
                bundle.index?.let { rows ->
                    zip.putNextEntry(ZipEntry(CSV))
                    zip.write(IndexFiles.csv(bundle = bundle, rows = rows))
                    zip.closeEntry()
                    zip.putNextEntry(ZipEntry(HTML))
                    zip.write(IndexFiles.html(bundle = bundle, rows = rows))
                    zip.closeEntry()
                }
            }
            if (!verified(zipFile = temp, sources = sources)) {
                return StorageIo.failure(reason = StorageFailure.VERIFICATION_FAILED)
            }
            Files.move(temp, file)
            return StorageResult.Success(value = file.toString())
        } finally {
            Files.deleteIfExists(temp)
        }
    }

    /** Reads every entry back and compares its hash with the original's. */
    private fun verified(zipFile: Path, sources: Map<String, Path>): Boolean {
        ZipFile(zipFile.toFile()).use { zip ->
            return sources.all { (name, source) ->
                val entry = zip.getEntry(name) ?: return@all false
                zip.getInputStream(entry).use { ContentHasher.hash(input = it) } == ContentHasher.hash(path = source)
            }
        }
    }

    /** [name] in [directory], or the first free `name (n)`; nothing existing is ever replaced. */
    private fun free(directory: Path, name: String): Path {
        val existing = Files.newDirectoryStream(directory).use { entries -> entries.map { it.fileName.toString() } }
        return directory.resolve(FileNamePolicy.nextFreeName(name = name, siblings = existing))
    }

    private fun sameFile(first: Path, second: Path): Boolean {
        return Files.exists(first) && Files.exists(second) && Files.isSameFile(first, second)
    }

    private companion object {
        const val CSV = "index.csv"
        const val HTML = "index.html"
    }
}
