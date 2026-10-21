package dms.domain

import dms.domain.catalog.Catalog
import dms.domain.catalog.CatalogUpdate
import dms.domain.catalog.LibraryId
import dms.domain.document.ContentHash
import dms.domain.document.Document
import dms.domain.document.DocumentId
import dms.domain.library.RelativePath
import dms.domain.taxonomy.CategoryId
import dms.domain.taxonomy.TagName
import java.time.Instant
import kotlin.test.assertIs

/** Synthetic test data. Never use real document names here. */
object TestData {

    val LIBRARY_ID: LibraryId = LibraryId(value = "library-1")
    val ADDED_AT: Instant = Instant.parse("2024-01-01T10:00:00Z")

    /** A valid content hash derived from [seed], so different seeds give different hashes. */
    fun hash(seed: Int): ContentHash {
        return ContentHash(value = seed.toString(radix = 16).padStart(length = 64, padChar = '0'))
    }

    fun document(
        id: String,
        path: String,
        hashSeed: Int = id.hashCode(),
        size: Long = 100,
        categoryId: CategoryId? = null,
        tags: Set<TagName> = emptySet(),
    ): Document {
        return Document(
            id = DocumentId(value = id),
            path = RelativePath.of(raw = path),
            contentHash = hash(seed = hashSeed and Int.MAX_VALUE),
            size = size,
            addedAt = ADDED_AT,
            categoryId = categoryId,
            tags = tags,
        )
    }

    fun catalogWith(vararg documents: Document): Catalog {
        return documents.fold(initial = Catalog.empty(libraryId = LIBRARY_ID)) { catalog, document ->
            catalog.addDocument(document = document).applied()
        }
    }

    /** Unwraps an applied update and fails the test for a rejection. */
    fun CatalogUpdate.applied(): Catalog {
        return assertIs<CatalogUpdate.Applied>(value = this).catalog
    }
}
