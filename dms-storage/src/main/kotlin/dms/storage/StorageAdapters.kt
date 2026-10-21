package dms.storage

import dms.storage.catalog.FileCatalogRepository
import dms.storage.fs.FileSystemDocumentStorage
import dms.storage.platform.PlatformTrash
import dms.storage.platform.SystemTrash
import dms.storage.scan.FileSystemLibraryScanner
import dms.storage.state.FileActivityLog
import dms.storage.state.FileOperationJournal
import dms.storage.state.FileSettingsRepository
import dms.storage.state.FileTaskStore
import dms.storage.text.FileTextStore
import java.nio.file.Path

/**
 * Every storage adapter, with the state of this device in [appDataDirectory]. The composition root
 * creates one instance and hands the adapters to the use cases.
 */
class StorageAdapters(
    appDataDirectory: Path,
    trash: SystemTrash = PlatformTrash(),
) {
    val documents: FileSystemDocumentStorage = FileSystemDocumentStorage(trash = trash)
    val catalogs: FileCatalogRepository = FileCatalogRepository(trash = trash)
    val scanner: FileSystemLibraryScanner =
        FileSystemLibraryScanner(cacheDirectory = appDataDirectory.resolve("scan-cache"))
    val texts: FileTextStore = FileTextStore()
    val journal: FileOperationJournal = FileOperationJournal(path = appDataDirectory.resolve("journal.yaml"))
    val tasks: FileTaskStore = FileTaskStore(path = appDataDirectory.resolve("tasks.yaml"))
    val activity: FileActivityLog = FileActivityLog(path = appDataDirectory.resolve("activity.yaml"))
    val settings: FileSettingsRepository = FileSettingsRepository(path = appDataDirectory.resolve("settings.yaml"))
}
