package dms.storage.fs

import dms.application.port.storage.model.FolderEntriesModel
import dms.domain.library.model.RelativePathModel
import dms.storage.scan.IgnoredEntries
import java.nio.file.FileVisitResult
import java.nio.file.Files
import java.nio.file.LinkOption
import java.nio.file.Path
import java.nio.file.SimpleFileVisitor
import java.nio.file.attribute.BasicFileAttributes

/** Walks a directory of a library folder for every folder and file below it, skipping what a scan skips. */
object FolderWalk {

    fun entries(root: Path, directory: Path): FolderEntriesModel {
        val folders = mutableSetOf<RelativePathModel>()
        val files = mutableSetOf<RelativePathModel>()
        Files.walkFileTree(
            directory,
            object : SimpleFileVisitor<Path>() {
                override fun preVisitDirectory(dir: Path, attrs: BasicFileAttributes): FileVisitResult {
                    if (dir == directory) {
                        return FileVisitResult.CONTINUE
                    }
                    if (isIgnored(root = root, entry = dir)) {
                        return FileVisitResult.SKIP_SUBTREE
                    }
                    folders += relativeOf(root = root, entry = dir)
                    return FileVisitResult.CONTINUE
                }

                override fun visitFile(file: Path, attrs: BasicFileAttributes): FileVisitResult {
                    if (!isIgnored(root = root, entry = file)) {
                        files += relativeOf(root = root, entry = file)
                    }
                    return FileVisitResult.CONTINUE
                }
            },
        )
        return FolderEntriesModel(folders = folders, files = files)
    }

    /** Every folder below [root], skipping what a scan skips; only directories are followed, files are not read. */
    fun folders(root: Path): Set<RelativePathModel> {
        val folders = mutableSetOf<RelativePathModel>()
        val pending = ArrayDeque(elements = listOf(root))
        while (pending.isNotEmpty()) {
            val directory = pending.removeFirst()
            Files.newDirectoryStream(directory) { Files.isDirectory(it, LinkOption.NOFOLLOW_LINKS) }.use { children ->
                children.filterNot { isIgnored(root = root, entry = it) }.forEach { child ->
                    folders += relativeOf(root = root, entry = child)
                    pending.addLast(element = child)
                }
            }
        }
        return folders
    }

    private fun isIgnored(root: Path, entry: Path): Boolean {
        return IgnoredEntries.isIgnored(name = PathResolver.nameOf(entry = entry), atRoot = entry.parent == root)
    }

    private fun relativeOf(root: Path, entry: Path): RelativePathModel {
        return RelativePathModel.fromSegments(
            segments = root.relativize(entry).map { PathResolver.nameOf(entry = it) },
        )
    }
}
