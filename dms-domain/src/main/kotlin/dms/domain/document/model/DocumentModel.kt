package dms.domain.document.model

import dms.domain.document.DocumentKind
import dms.domain.field.FieldValue
import dms.domain.field.model.FieldKeyModel
import dms.domain.library.model.RelativePathModel
import dms.domain.retention.model.RetentionModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.TagNameModel
import java.time.Instant

/**
 * A file in the library together with everything the catalog knows about it: metadata, category,
 * tags, custom [fields], a free-text [note] and an opt-in [retention].
 */
data class DocumentModel(
    val id: DocumentIdModel,
    val path: RelativePathModel,
    val contentHash: ContentHashModel,
    val size: Long,
    val addedAt: Instant,
    val metadata: DocumentMetadataModel = DocumentMetadataModel.EMPTY,
    val categoryId: CategoryIdModel? = null,
    val tags: Set<TagNameModel> = emptySet(),
    val classification: ClassificationRecordModel? = null,
    val fields: Map<FieldKeyModel, FieldValue> = emptyMap(),
    val note: String? = null,
    val retention: RetentionModel? = null,
) {
    init {
        require(value = !path.isRoot) { "A document needs a file path" }
        require(value = size >= 0) { "Size must not be negative" }
        require(value = note == null || note.isNotBlank()) { "Note must be null or not blank" }
        require(value = retention == null || !retention.isEmpty) { "An empty retention is stored as none" }
    }

    val kind: DocumentKind
        get() {
            return DocumentKind.of(path = path)
        }
}
