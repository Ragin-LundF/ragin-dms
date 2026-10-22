package dms.application.search

import dms.application.TestLibrary
import dms.application.catalog.model.CatalogEdit
import dms.domain.taxonomy.model.TagNameModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals

class SavedSearchPresetsTest {

    @Test
    fun `the presets find untagged, uncategorized and this year's tax documents`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val tax = TagNameModel.of(raw = "Tax")
            val plain = library.add(path = "plain.pdf", content = "a")
            val taxed = library.add(path = "2024-02-01 tax.pdf", content = "b")
            library.editCatalog(edit = CatalogEdit.AddTags(documentId = taxed.id, tags = setOf(tax)))

            val presets = SavedSearchPresets.of(today = library.clock.today(), ids = library.ids, taxTag = tax)
            val found = presets.associate { preset ->
                preset.name to library.searchContents.search(query = preset.query).map { it.id }
            }

            assertEquals(
                expected = mapOf(
                    "Untagged" to listOf(plain.id),
                    "Uncategorized" to listOf(taxed.id, plain.id),
                    "Tax 2024" to listOf(taxed.id),
                    "Due in 30 days" to emptyList(),
                ),
                actual = found,
            )
        }
    }
}
