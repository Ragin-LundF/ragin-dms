package dms.domain.document

import dms.domain.library.RelativePath
import dms.domain.taxonomy.CategoryId
import dms.domain.taxonomy.TagName
import java.time.Instant

/** A file in the library together with everything the catalog knows about it. */
data class Document(
    val id: DocumentId,
    val path: RelativePath,
    val contentHash: ContentHash,
    val size: Long,
    val addedAt: Instant,
    val metadata: DocumentMetadata = DocumentMetadata.EMPTY,
    val categoryId: CategoryId? = null,
    val tags: Set<TagName> = emptySet(),
    val classification: ClassificationRecord? = null,
) {
    init {
        require(value = !path.isRoot) { "A document needs a file path" }
        require(value = size >= 0) { "Size must not be negative" }
    }

    val kind: DocumentKind
        get() {
            return DocumentKind.of(path = path)
        }
}
