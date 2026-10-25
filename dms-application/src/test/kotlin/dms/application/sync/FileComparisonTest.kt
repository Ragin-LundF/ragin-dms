package dms.application.sync

import dms.application.TestLibrary
import dms.application.fake.storage.FakeFolders
import dms.domain.catalog.CatalogUpdate
import dms.domain.catalog.model.CatalogModel
import dms.domain.catalog.model.LibraryIdModel
import dms.domain.change.model.FileFactsModel
import dms.domain.library.model.RelativePathModel
import java.text.Normalizer
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotEquals
import kotlin.test.assertNull

class FileComparisonTest {

    private val path = RelativePathModel.of(raw = "letters/notice.pdf")
    private val catalog = catalogWith(path = "letters/notice.pdf", content = "catalogued")
    private val catalogued = FakeFolders.factsOf(content = "catalogued")
    private val composed = "Bel\u00e4ge/\u00fcbersicht.pdf"
    private val decomposed = Normalizer.normalize(composed, Normalizer.Form.NFD)

    @Test
    fun `the hash the scanner computed wins over the catalogued one`() {
        val facts = FakeFolders.factsOf(content = "changed")

        val hash = FileComparison.effectiveHash(facts = facts, path = path, catalog = catalog)

        assertEquals(expected = FakeFolders.hashOf(content = "changed"), actual = hash)
    }

    @Test
    fun `a file the scanner did not hash has the catalogued content`() {
        val hash = FileComparison.effectiveHash(facts = unhashed(facts = catalogued), path = path, catalog = catalog)

        assertEquals(expected = FakeFolders.hashOf(content = "catalogued"), actual = hash)
    }

    @Test
    fun `an unhashed file the catalog does not know has no hash`() {
        val hash = FileComparison.effectiveHash(
            facts = unhashed(facts = catalogued),
            path = RelativePathModel.of(raw = "letters/other.pdf"),
            catalog = catalog,
        )

        assertNull(actual = hash)
    }

    @Test
    fun `a decomposed path finds the catalogued hash of its composed form`() {
        val composedCatalog = catalogWith(path = composed, content = "catalogued")

        val hash = FileComparison.effectiveHash(
            facts = unhashed(facts = catalogued),
            path = RelativePathModel.of(raw = decomposed),
            catalog = composedCatalog,
        )

        assertNotEquals(illegal = composed, actual = decomposed)
        assertEquals(expected = FakeFolders.hashOf(content = "catalogued"), actual = hash)
    }

    @Test
    fun `two copies with the same hash do not differ`() {
        val differs = FileComparison.differs(
            first = FakeFolders.factsOf(content = "same"),
            second = FakeFolders.factsOf(content = "same"),
            path = path,
            catalog = catalog,
        )

        assertEquals(expected = false, actual = differs)
    }

    @Test
    fun `copies of the same size with different content differ`() {
        val differs = FileComparison.differs(
            first = FakeFolders.factsOf(content = "aaaa"),
            second = FakeFolders.factsOf(content = "bbbb"),
            path = path,
            catalog = catalog,
        )

        assertEquals(expected = true, actual = differs)
    }

    @Test
    fun `equal hashes decide even when the reported sizes disagree`() {
        val first = FakeFolders.factsOf(content = "same")

        val differs = FileComparison.differs(
            first = first,
            second = first.copy(size = first.size + 1),
            path = path,
            catalog = catalog,
        )

        assertEquals(expected = false, actual = differs)
    }

    @Test
    fun `two unhashed copies of a catalogued file both have the catalogued content`() {
        val differs = FileComparison.differs(
            first = unhashed(facts = catalogued),
            second = unhashed(facts = catalogued),
            path = path,
            catalog = catalog,
        )

        assertEquals(expected = false, actual = differs)
    }

    @Test
    fun `an unhashed copy matches a hashed copy with the catalogued content`() {
        val differs = FileComparison.differs(
            first = unhashed(facts = catalogued),
            second = catalogued,
            path = path,
            catalog = catalog,
        )

        assertEquals(expected = false, actual = differs)
    }

    @Test
    fun `a changed copy of the same size differs from an unhashed catalogued copy`() {
        val changed = FakeFolders.factsOf(content = "CATALOGUED")

        val differs = FileComparison.differs(
            first = unhashed(facts = catalogued),
            second = changed,
            path = path,
            catalog = catalog,
        )

        assertEquals(expected = catalogued.size, actual = changed.size)
        assertEquals(expected = true, actual = differs)
    }

    @Test
    fun `a content conflict is found under a decomposed path although the sizes match`() {
        val composedCatalog = catalogWith(path = composed, content = "catalogued")

        val differs = FileComparison.differs(
            first = unhashed(facts = catalogued),
            second = FakeFolders.factsOf(content = "CATALOGUED"),
            path = RelativePathModel.of(raw = decomposed),
            catalog = composedCatalog,
        )

        assertEquals(expected = true, actual = differs)
    }

    @Test
    fun `without a hash on one side an unknown file falls back to its size`() {
        val other = RelativePathModel.of(raw = "letters/other.pdf")

        val sameSize = FileComparison.differs(
            first = unhashed(facts = FakeFolders.factsOf(content = "aaaa")),
            second = FakeFolders.factsOf(content = "bbbb"),
            path = other,
            catalog = catalog,
        )
        val otherSize = FileComparison.differs(
            first = unhashed(facts = FakeFolders.factsOf(content = "aaaa")),
            second = FakeFolders.factsOf(content = "bbbbb"),
            path = other,
            catalog = catalog,
        )

        assertEquals(expected = false, actual = sameSize)
        assertEquals(expected = true, actual = otherSize)
    }

    @Test
    fun `without any hash an unknown file falls back to its size`() {
        val other = RelativePathModel.of(raw = "letters/other.pdf")

        val sameSize = FileComparison.differs(
            first = FileFactsModel(size = 10, contentHash = null),
            second = FileFactsModel(size = 10, contentHash = null),
            path = other,
            catalog = catalog,
        )
        val otherSize = FileComparison.differs(
            first = FileFactsModel(size = 10, contentHash = null),
            second = FileFactsModel(size = 11, contentHash = null),
            path = other,
            catalog = catalog,
        )

        assertEquals(expected = false, actual = sameSize)
        assertEquals(expected = true, actual = otherSize)
    }

    private fun unhashed(facts: FileFactsModel): FileFactsModel {
        return facts.copy(contentHash = null)
    }

    private fun catalogWith(path: String, content: String): CatalogModel {
        val empty = CatalogModel.empty(libraryId = LibraryIdModel(value = "library"))
        val update = empty.addDocument(document = TestLibrary.document(id = "doc-1", path = path, content = content))
        return assertIs<CatalogUpdate.Applied>(value = update).catalog
    }
}
