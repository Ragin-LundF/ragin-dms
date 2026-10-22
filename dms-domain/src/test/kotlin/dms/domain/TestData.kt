package dms.domain

import dms.domain.catalog.CatalogUpdate
import dms.domain.catalog.model.CatalogModel
import dms.domain.catalog.model.LibraryIdModel
import dms.domain.document.model.ContentHashModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.TagNameModel
import java.time.Instant
import kotlin.test.assertIs

/** Synthetic test data. Never use real document names here. */
object TestData {

    val LIBRARY_ID: LibraryIdModel = LibraryIdModel(value = "library-1")
    val ADDED_AT: Instant = Instant.parse("2024-01-01T10:00:00Z")

    /** A valid content hash derived from [seed], so different seeds give different hashes. */
    fun hash(seed: Int): ContentHashModel {
        return ContentHashModel(value = seed.toString(radix = 16).padStart(length = 64, padChar = '0'))
    }

    fun document(
        id: String,
        path: String,
        hashSeed: Int = id.hashCode(),
        size: Long = 100,
        categoryId: CategoryIdModel? = null,
        tags: Set<TagNameModel> = emptySet(),
    ): DocumentModel {
        return DocumentModel(
            id = DocumentIdModel(value = id),
            path = RelativePathModel.of(raw = path),
            contentHash = hash(seed = hashSeed and Int.MAX_VALUE),
            size = size,
            addedAt = ADDED_AT,
            categoryId = categoryId,
            tags = tags,
        )
    }

    fun catalogWith(vararg documents: DocumentModel): CatalogModel {
        return documents.fold(initial = CatalogModel.empty(libraryId = LIBRARY_ID)) { catalog, document ->
            catalog.addDocument(document = document).applied()
        }
    }

    /** Unwraps an applied update and fails the test for a rejection. */
    fun CatalogUpdate.applied(): CatalogModel {
        return assertIs<CatalogUpdate.Applied>(value = this).catalog
    }
}
