package dms.storage.export

import dms.application.port.export.model.ExportBundleModel
import dms.application.port.export.model.ExportEntryModel
import dms.application.port.export.model.ExportTargetModel
import dms.application.port.export.model.IndexRowModel
import dms.storage.support.StorageResults
import dms.storage.support.TestFolders
import kotlinx.coroutines.test.runTest
import java.nio.charset.StandardCharsets
import java.nio.file.Files
import java.nio.file.Path
import java.util.zip.ZipFile
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class FileExportWriterTest {

    private val temp: Path = Files.createTempDirectory("dms-export-")
    private val root: Path = Files.createDirectories(temp.resolve("library"))
    private val outside: Path = Files.createDirectories(temp.resolve("outside"))
    private val folder = TestFolders.folder(location = root)
    private val writer = FileExportWriter()

    private val row = IndexRowModel(
        path = "letters/Grüße.pdf",
        title = "Bill; second \"try\"",
        date = "2024-02-01",
        correspondent = "Müller & Söhne",
        category = "Bills / Power",
        tags = "Tax, Home",
        fields = listOf("12.50 EUR"),
        note = "first line\nsecond line",
    )
    private val bundle = ExportBundleModel(
        entries = listOf(
            ExportEntryModel(source = TestFolders.path(raw = "a/Grüße.pdf"), target = "letters/Grüße.pdf"),
        ),
        index = listOf(row),
        fieldLabels = listOf("Amount"),
    )

    @AfterTest
    fun deleteTemp() {
        temp.toFile().deleteRecursively()
    }

    @Test
    fun `a folder export copies the files and writes both index files, the library stays untouched`() {
        runTest {
            TestFolders.write(file = root.resolve("a/Grüße.pdf"), content = "content")
            val before = Files.readAllBytes(root.resolve("a/Grüße.pdf"))

            val location = StorageResults.valueOf(
                result = writer.write(
                    source = folder,
                    bundle = bundle,
                    target = ExportTargetModel(directory = outside.toString(), name = "Tax 2024", zip = false),
                ),
            )

            val exported = Path.of(location)
            assertEquals(expected = outside.resolve("Tax 2024"), actual = exported)
            assertEquals(expected = "content", actual = Files.readString(exported.resolve("letters/Grüße.pdf")))
            assertTrue(actual = Files.exists(exported.resolve("index.html")))
            assertContentEquals(expected = before, actual = Files.readAllBytes(root.resolve("a/Grüße.pdf")))
            val names = Files.walk(root).use { paths ->
                paths.filter { it != root }.map { it.fileName.toString() }.toList()
            }
            assertEquals(expected = setOf("a", "Grüße.pdf"), actual = names.toSet())
        }
    }

    @Test
    fun `the CSV keeps umlauts and quotes fields with semicolons, quotes and line breaks`() {
        runTest {
            val csv = IndexFiles.csv(bundle = bundle, rows = listOf(row))

            val text = String(csv, StandardCharsets.UTF_8)
            assertEquals(expected = '﻿', actual = text.first())
            val lines = text.removePrefix(prefix = "﻿")
            assertTrue(
                actual = lines.startsWith(prefix = "Path;Title;Date;Correspondent;Category;Tags;Amount;Note\r\n"),
            )
            assertTrue(
                actual = lines.contains(
                    other = "letters/Grüße.pdf;" +
                        "\"Bill; second \"\"try\"\"\";2024-02-01;Müller & Söhne;Bills / Power;" +
                        "\"Tax, Home\";12.50 EUR;\"first line\nsecond line\"\r\n",
                ),
            )
        }
    }

    @Test
    fun `a ZIP export holds the files and the index, and an existing name is never replaced`() {
        runTest {
            TestFolders.write(file = root.resolve("a/Grüße.pdf"), content = "content")
            TestFolders.write(file = outside.resolve("Tax.zip"), content = "older export")
            val target = ExportTargetModel(directory = outside.toString(), name = "Tax", zip = true)

            val location = StorageResults.valueOf(
                result = writer.write(source = folder, bundle = bundle, target = target),
            )

            assertEquals(expected = outside.resolve("Tax (2).zip").toString(), actual = location)
            assertEquals(expected = "older export", actual = Files.readString(outside.resolve("Tax.zip")))
            ZipFile(location).use { zip ->
                assertEquals(
                    expected = setOf("letters/Grüße.pdf", "index.csv", "index.html"),
                    actual = zip.entries().toList().map { it.name }.toSet(),
                )
                val content = zip.getInputStream(zip.getEntry("letters/Grüße.pdf"))
                    .use { String(it.readBytes(), StandardCharsets.UTF_8) }
                assertEquals(expected = "content", actual = content)
            }
            assertFalse(
                actual = Files.list(outside).use { files ->
                    files.anyMatch { it.fileName.toString().startsWith(prefix = ".dms-tmp-") }
                },
            )
        }
    }

    @Test
    fun `a target inside a library folder is recognized`() {
        assertTrue(actual = writer.isInside(directory = root.resolve("exports").toString(), folders = listOf(folder)))
        assertTrue(actual = writer.isInside(directory = root.toString(), folders = listOf(folder)))
        assertFalse(actual = writer.isInside(directory = outside.toString(), folders = listOf(folder)))
    }

    @Test
    fun `a text file such as a calendar is written under a free name`() {
        runTest {
            TestFolders.write(file = outside.resolve("deadlines.ics"), content = "old")

            val location = StorageResults.valueOf(
                result = writer.writeText(
                    directory = outside.toString(),
                    name = "deadlines.ics",
                    text = "BEGIN:VCALENDAR",
                ),
            )

            assertEquals(expected = "BEGIN:VCALENDAR", actual = Files.readString(Path.of(location)))
            assertEquals(expected = "old", actual = Files.readString(outside.resolve("deadlines.ics")))
        }
    }
}
