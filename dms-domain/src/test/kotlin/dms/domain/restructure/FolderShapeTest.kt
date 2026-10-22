package dms.domain.restructure

import dms.domain.TestData
import dms.domain.TestData.applied
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.library.model.RelativePathModel
import dms.domain.restructure.model.FlatFolderModel
import dms.domain.settings.model.InboxSettingsModel
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class FolderShapeTest {

    private fun catalogWith(folder: String, count: Int, dated: Boolean = false): CatalogModel {
        return (1..count).fold(initial = TestData.catalogWith()) { catalog, index ->
            val document = TestData.document(id = "$folder-$index", path = "$folder/file-$index.pdf")
            val added = catalog.addDocument(document = document).applied()
            if (dated) {
                val date = LocalDate.of(2020 + index % 3, 1, 1)
                added.updateMetadata(
                    documentId = document.id,
                    metadata = DocumentMetadataModel(documentDate = date),
                ).applied()
            } else {
                added
            }
        }
    }

    @Test
    fun `a folder with more files than the threshold and no subfolders is flat`() {
        val catalog = catalogWith(folder = "scans", count = 41)

        assertEquals(
            expected = listOf(FlatFolderModel(path = RelativePathModel.of(raw = "scans"), fileCount = 41)),
            actual = FolderShape.analyse(catalog = catalog, inbox = InboxSettingsModel()),
        )
    }

    @Test
    fun `a folder with subfolders is not flat`() {
        val catalog = catalogWith(folder = "scans", count = 41)
            .addDocument(document = TestData.document(id = "deep", path = "scans/2024/deep.pdf")).applied()

        assertTrue(actual = FolderShape.analyse(catalog = catalog, inbox = InboxSettingsModel()).isEmpty())
    }

    @Test
    fun `a smaller folder that splits cleanly by year is offered`() {
        val catalog = catalogWith(folder = "letters", count = 12, dated = true)

        assertEquals(expected = 1, actual = FolderShape.analyse(catalog = catalog, inbox = InboxSettingsModel()).size)
        assertTrue(
            actual = FolderShape.analyse(
                catalog = catalogWith(folder = "letters", count = 12),
                inbox = InboxSettingsModel(),
            ).isEmpty(),
        )
    }

    @Test
    fun `the inbox is never offered`() {
        val catalog = catalogWith(folder = "Inbox", count = 50)

        assertTrue(
            actual = FolderShape.analyse(catalog = catalog, inbox = InboxSettingsModel(enabled = true)).isEmpty(),
        )
    }
}
