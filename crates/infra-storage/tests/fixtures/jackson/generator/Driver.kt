// Generates the Jackson golden files for dms-infra-storage from the released Kotlin 1.3.0 jars.
// Only synthetic inputs; everything runs in temporary directories. Not part of the cargo build.
import dms.application.port.export.model.ExportBundleModel
import dms.application.port.export.model.ExportEntryModel
import dms.application.port.export.model.IndexRowModel
import dms.application.port.index.model.DocumentVectorsModel
import dms.application.port.index.model.IndexedChunkModel
import dms.application.port.plan.model.RestructureRecordModel
import dms.application.port.state.model.ActivityEntryModel
import dms.application.port.state.model.ActivityKind
import dms.application.port.state.model.AiProvider
import dms.application.port.state.model.AiSettingsModel
import dms.application.port.state.model.LayoutPreferencesModel
import dms.application.port.state.model.PreferencesModel
import dms.application.port.storage.model.CatalogLoad
import dms.application.port.storage.model.FolderOperation
import dms.application.port.storage.model.PendingOperationModel
import dms.domain.catalog.model.CatalogModel
import dms.domain.catalog.model.LibraryIdModel
import dms.domain.classification.SuggestedCategory
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.classification.model.SuggestedTagModel
import dms.domain.document.model.ContentHashModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.FolderRole
import dms.domain.library.model.FolderLocationModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.LibraryFoldersModel
import dms.domain.library.model.RelativePathModel
import dms.domain.restructure.model.PlannedMoveModel
import dms.domain.task.model.TaskKeyModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.TagNameModel
import dms.storage.catalog.FileCatalogRepository
import dms.storage.export.IndexFiles
import dms.storage.index.FileVectorIndex
import dms.storage.plan.FilePlanStore
import dms.storage.scan.FileSystemLibraryScanner
import dms.storage.state.FileActivityLog
import dms.storage.state.FileOperationJournal
import dms.storage.state.FileSettingsRepository
import dms.storage.state.FileTaskStore
import dms.storage.yaml.YamlMappers
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.attribute.FileTime
import java.time.Instant
import java.time.LocalDate
import kotlinx.coroutines.runBlocking

fun main(args: Array<String>) {
    val inputs = Path.of(args[0])
    val out = Files.createDirectories(Path.of(args[1]))
    val temp = Files.createTempDirectory("dms-golden-")
    runBlocking {
        scalars(out)
        catalogs(inputs, out, temp)
        state(out, temp)
        folderData(out, temp)
    }
}

fun folder(location: Path, id: String = "primary") = LibraryFolderModel(
    id = LibraryFolderIdModel(value = id),
    displayName = id,
    location = FolderLocationModel(value = location.toString()),
    role = FolderRole.PRIMARY,
)

fun scalars(out: Path) {
    val values = listOf(
        "plain", "", " ", "yes", "Yes", "YES", "no", "No", "on", "Off", "y", "n", "true", "False", "null", "Null", "~",
        "2024", "0123", "-5", "+5", "1.5", ".5", "5.", "1e3", "1E+3", ".inf", "-.inf", ".nan", "0x1F", "12_34", "1:30",
        "2024-01-01", "2024-01-01T10:00:00Z", "-", "- a", "-a", "a - b", "a: b", "a:b", "a:", ":a", "? a", "?a", "a?",
        "#tag", "a #b", "a#b", "[x]", "a[x]", "{x}", "a,b", "&anchor", "*star", "!bang", "|pipe", ">gt", "'q'",
        "\"dq\"", "%pct", "@at", "`tick", "---x", "...x", "a---", "äöü ß", "emoji 😀", "line\u2028sep", "café",
        "nul\u0000x", "nel\u0085x", "nbsp\u00a0x", "€", "back\\slash", "bell\u0007x", "tab\tin", "cr\rx", "bom\ufeffx",
        "private\ue000x", "noncharacter\uffffx", " leading", "trailing ", "a  b",
        "multi\nline", "trailing\n", "\nleading", "keep\n\n", "a\n\nb", "a\n b", "a \nb", " indented\nsecond",
        "tab\tand\nline", "cr\r\nline", "x".repeat(100),
        "This is a long sentence that should be wrapped by the emitter because it is longer than eighty columns.",
        "#This is a long sentence that has to be quoted and should be wrapped because it is longer than eighty.",
        "This is a long sentence: it has to be single quoted and it should wrap since it is longer than eighty.",
        "This is a long sentence with a tab\there and it is double quoted and it should wrap after eighty columns",
        "Thisisalongwordwithoutspaces Thatisfollowedbyanotherlongwordwithoutspaces Andonemorelongwordafterthat",
        "two  spaces  between  words  in  a  long  sentence  that  goes  beyond  the  eighty  columns  limit  yes",
        "ümlaut äöü ümlaut äöü ümlaut äöü ümlaut äöü ümlaut äöü ümlaut äöü ümlaut äöü ümlaut äöü ümlaut äöü end",
        "😀😀😀😀😀😀😀😀😀😀 😀😀😀😀😀😀😀😀😀😀 😀😀😀😀😀😀😀😀😀😀 😀😀😀😀😀😀😀😀😀😀 😀😀😀😀😀😀😀😀😀😀 end",
        "A long line that is followed by\na second line that is also very long and goes beyond eighty columns here",
    )
    val map = linkedMapOf<String, Any?>()
    values.forEachIndexed { index, value -> map["v%03d".format(index)] = value }
    map["list"] = values
    map["nested"] = linkedMapOf("deeper" to linkedMapOf("value" to values.last(), "list" to listOf(values.last())))
    val keys = listOf(
        "2024/a.pdf", "a: b.pdf", "yes", "no.pdf", "#x", "-a", "- a", "tab\tkey", "ä.pdf", "a,b", "[x]", "",
        "k".repeat(127), "k".repeat(128), "line\nkey", "~", "null", ".hidden", "+x", "a'b", "😀.pdf", " lead",
    )
    keys.forEach { map[it] = "value" }
    map["emptyList"] = emptyList<String>()
    map["emptyMap"] = emptyMap<String, String>()
    map["nullValue"] = null
    map["int"] = 5
    map["long"] = 5_000_000_000L
    map["negative"] = -3
    map["bool"] = true
    map["double"] = 0.75
    map["doubleSmall"] = 1.0E-4
    map["doubleBig"] = 1.0E7
    map["doubleMid"] = 123456.789
    map["doubleOne"] = 1.0
    map["listOfLists"] = listOf(listOf("a", "b"), emptyList<String>(), listOf(linkedMapOf("k" to "v")))
    map["listOfMaps"] = listOf(linkedMapOf("a" to "1", "b" to listOf("x")), emptyMap<String, String>())
    Files.write(out.resolve("scalars.yaml"), YamlMappers.yaml.writeValueAsBytes(map))
}

suspend fun catalogs(inputs: Path, out: Path, temp: Path) {
    for (name in listOf("odd-catalog.yaml", "future-fields.yaml", "format-1-v1.0.yaml", "format-1-v1.1.yaml")) {
        val root = Files.createDirectories(temp.resolve("catalog-$name"))
        val file = root.resolve("dms-catalog.yaml")
        Files.copy(inputs.resolve(name), file)
        val repository = FileCatalogRepository(trash = { _, _ -> false })
        val load = repository.load(folder = folder(root))
        val catalog = (load as? CatalogLoad.Found)?.catalog ?: error("$name: $load")
        val saved = repository.save(folder = folder(root), catalog = catalog, basedOn = catalog.revision)
        println("$name: $saved")
        Files.copy(file, out.resolve("saved-$name"))
    }
}

suspend fun state(out: Path, temp: Path) {
    val settings = FileSettingsRepository(path = out.resolve("settings-default.yaml"))
    settings.saveLibraryFolders(
        folders = LibraryFoldersModel(
            folders = listOf(
                LibraryFolderModel(
                    id = LibraryFolderIdModel(value = "primary"),
                    displayName = "Bibliothek: Haupt",
                    location = FolderLocationModel(value = "/data/Bibliothek 2024"),
                    role = FolderRole.PRIMARY,
                ),
                LibraryFolderModel(
                    id = LibraryFolderIdModel(value = "sync"),
                    displayName = "yes",
                    location = FolderLocationModel(value = "C:\\Users\\Example\\Library"),
                    role = FolderRole.SYNC,
                ),
            ),
        ),
    )
    val full = FileSettingsRepository(path = out.resolve("settings-full.yaml"))
    full.savePreferences(
        preferences = PreferencesModel(
            addNewFilesWhileScanning = false,
            ai = AiSettingsModel(
                enabled = true,
                baseUrl = "http://localhost:9999/v1",
                model = "sample-model",
                preferVisionForPdfs = true,
                acceptSuggestedFileNames = true,
                embeddingModel = "embed: v2",
                inputCharacters = 30_000,
                provider = AiProvider.ANTHROPIC,
                anthropicModel = "claude-sonnet-5-5",
            ),
            layout = LayoutPreferencesModel(sidebarWidth = 280, sidebarFolded = true, folderTreeWidth = 300),
        ),
    )

    val tasks = FileTaskStore(path = out.resolve("tasks.yaml"))
    tasks.saveSuggestion(
        suggestion = ClassificationSuggestionModel(
            documentId = DocumentIdModel(value = "doc-1"),
            model = "test-model",
            title = "Sample: invoice",
            category = SuggestedCategory.New(name = "Rechnungen", parentId = CategoryIdModel(value = "cat-1")),
            tags = listOf(
                SuggestedTagModel(name = TagNameModel.of(raw = "Tax"), isNew = false),
                SuggestedTagModel(name = TagNameModel.of(raw = "2024"), isNew = true),
            ),
            documentDate = LocalDate.parse("2023-12-24"),
            correspondent = "Example Supplier",
            fileName = "2023-12-24 Sample invoice.pdf",
            transcription = "Line one\nLine two\n",
            confidence = 0.75,
        ),
    )
    tasks.saveSuggestion(
        suggestion = ClassificationSuggestionModel(
            documentId = DocumentIdModel(value = "doc-2"),
            model = "rules",
            category = SuggestedCategory.Existing(categoryId = CategoryIdModel(value = "cat-1")),
            confidence = 1.0E-4,
        ),
    )
    tasks.saveSuggestion(
        suggestion = ClassificationSuggestionModel(documentId = DocumentIdModel(value = "doc-3"), model = "m", confidence = 1.0),
    )
    tasks.dismiss(key = TaskKeyModel(value = "new:2024/a.pdf"))
    tasks.dismiss(key = TaskKeyModel(value = "missing:doc-1"))
    FileTaskStore(path = out.resolve("tasks-empty.yaml")).removeSuggestion(documentId = DocumentIdModel(value = "x"))

    val activity = FileActivityLog(path = out.resolve("activity.yaml"))
    activity.record(ActivityEntryModel(Instant.parse("2024-05-01T08:00:00Z"), ActivityKind.DOCUMENT_ADDED, "a: b.pdf"))
    activity.record(ActivityEntryModel(Instant.parse("2024-05-01T09:00:00.5Z"), ActivityKind.PDF_EDITED, "2024"))

    val journal = FileOperationJournal(path = out.resolve("journal.yaml"))
    listOf(
        FolderOperation.CopyFromPrimary(path = RelativePathModel.of(raw = "a.pdf"), replaceExisting = true),
        FolderOperation.CopyFromPrimary(path = RelativePathModel.of(raw = "café/b.pdf")),
        FolderOperation.CreateFolder(path = RelativePathModel.of(raw = "letters")),
        FolderOperation.Move(from = RelativePathModel.of(raw = "a.pdf"), to = RelativePathModel.of(raw = "x/a.pdf")),
        FolderOperation.MoveToTrash(path = RelativePathModel.of(raw = "old.pdf")),
        FolderOperation.CopyText(documentId = DocumentIdModel(value = "doc-1")),
        FolderOperation.CopyPlan(name = "20261022-080000"),
        FolderOperation.CopyIndex(documentId = DocumentIdModel(value = "doc-2")),
        FolderOperation.SaveCatalog,
    ).forEachIndexed { index, operation ->
        journal.record(
            PendingOperationModel(
                id = "op-$index",
                folderId = LibraryFolderIdModel(value = "sync"),
                operation = operation,
                recordedAt = Instant.parse("2024-05-01T08:00:00Z"),
            ),
        )
    }

    val scanRoot = Files.createDirectories(temp.resolve("scan"))
    for (name in listOf("a.pdf", "2024/b.pdf", "yes.pdf", "Grüße.pdf", "Re: b.pdf")) {
        val file = scanRoot.resolve(name)
        Files.createDirectories(file.parent)
        Files.writeString(file, "content of $name")
        Files.setLastModifiedTime(file, FileTime.from(Instant.parse("2024-05-01T08:00:00Z")))
    }
    val scanner = FileSystemLibraryScanner(cacheDirectory = out.resolve("scan-cache"))
    println(scanner.scan(folder = folder(scanRoot), catalog = CatalogModel.empty(libraryId = LibraryIdModel(value = "l")), listener = object : dms.application.port.storage.ScanListener {}))
}

suspend fun folderData(out: Path, temp: Path) {
    val root = Files.createDirectories(temp.resolve("library"))
    FilePlanStore().write(
        folder = folder(root),
        record = RestructureRecordModel(
            name = "20261022-080000",
            appliedAt = Instant.parse("2026-10-22T08:00:00Z"),
            template = "{category}/{year}",
            moves = listOf(
                PlannedMoveModel(
                    documentId = DocumentIdModel(value = "doc-1"),
                    from = RelativePathModel.of(raw = "loose/Grüße.pdf"),
                    to = RelativePathModel.of(raw = "Letters/2024/Grüße.pdf"),
                ),
                PlannedMoveModel(
                    documentId = DocumentIdModel(value = "doc-2"),
                    from = RelativePathModel.of(raw = "2024.pdf"),
                    to = RelativePathModel.of(raw = "yes/2024.pdf"),
                ),
            ),
        ),
    )
    Files.copy(root.resolve(".dms-plans/20261022-080000.yaml"), out.resolve("plan.yaml"))
    FileVectorIndex().write(
        folder = folder(root),
        vectors = DocumentVectorsModel(
            documentId = DocumentIdModel(value = "doc-1"),
            model = "embed-ä\u0000😀",
            dimensions = 3,
            contentHash = ContentHashModel(value = "8ed3f6ad685b959ead7022518e1af76cd816f8e8ec7ccdda1ed4018e8f2223f8"),
            metadataHash = "1a2b",
            chunks = listOf(
                IndexedChunkModel(start = 0, end = 10, vector = floatArrayOf(0.1f, -0.2f, 3.5e-8f)),
                IndexedChunkModel(start = 8, end = 20, vector = floatArrayOf(-1f, 0f, Float.MAX_VALUE)),
            ),
        ),
    )
    Files.copy(root.resolve(".dms-index/doc-1.vec"), out.resolve("doc-1.vec"))
    val row = IndexRowModel(
        path = "letters/Grüße & <Söhne>.pdf",
        title = "Bill; second \"try\"",
        date = "2024-02-01",
        correspondent = "Müller & Söhne",
        category = "Bills / Power",
        tags = "Tax, Home",
        fields = listOf("12.50 EUR", ""),
        note = "first line\nsecond line\r\nthird",
    )
    val bundle = ExportBundleModel(
        entries = listOf(ExportEntryModel(source = RelativePathModel.of(raw = "a.pdf"), target = row.path)),
        index = listOf(row, row.copy(path = "a b+c/ä?.pdf", note = "")),
        fieldLabels = listOf("Amount", "Due; date"),
    )
    Files.write(out.resolve("index.csv"), IndexFiles.csv(bundle = bundle, rows = bundle.index!!))
    Files.write(out.resolve("index.html"), IndexFiles.html(bundle = bundle, rows = bundle.index!!))
}
