package dms.application.fake.export

import dms.application.port.export.ExportWriter
import dms.application.port.export.model.ExportBundleModel
import dms.application.port.export.model.ExportTargetModel
import dms.application.port.storage.model.StorageResult
import dms.domain.library.model.LibraryFolderModel

/** Records what would be written; every library folder location counts as inside the library. */
class FakeExportWriter : ExportWriter {

    val written: MutableList<Pair<ExportBundleModel, ExportTargetModel>> = mutableListOf()
    val texts: MutableMap<String, String> = mutableMapOf()

    override fun isInside(directory: String, folders: List<LibraryFolderModel>): Boolean {
        return folders.any { directory == it.location.value || directory.startsWith(prefix = it.location.value + "/") }
    }

    override suspend fun write(
        source: LibraryFolderModel,
        bundle: ExportBundleModel,
        target: ExportTargetModel,
    ): StorageResult<String> {
        written += bundle to target
        return StorageResult.Success(value = "${target.directory}/${target.name}")
    }

    override suspend fun writeText(directory: String, name: String, text: String): StorageResult<String> {
        texts["$directory/$name"] = text
        return StorageResult.Success(value = "$directory/$name")
    }
}
