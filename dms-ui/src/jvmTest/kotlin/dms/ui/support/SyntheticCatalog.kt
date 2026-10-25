package dms.ui.support

import dms.domain.catalog.model.CatalogModel
import dms.domain.catalog.model.CatalogRevisionModel
import dms.domain.catalog.model.LibraryIdModel
import dms.domain.document.model.ContentHashModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.document.model.DocumentModel
import dms.domain.field.FieldValue
import dms.domain.field.model.FieldKeyModel
import dms.domain.library.model.RelativePathModel
import dms.domain.settings.model.CatalogSettingsModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import dms.domain.taxonomy.model.TagNameModel
import java.time.Instant
import java.time.LocalDate

/** Catalogs built in memory for the pure functions of the library, with invented documents only. */
object SyntheticCatalog {

    val invoices: CategoryModel = CategoryModel(id = CategoryIdModel(value = "invoices"), name = "Invoices")
    val energy: CategoryModel = CategoryModel(
        id = CategoryIdModel(value = "energy"),
        name = "Energy",
        parentId = invoices.id,
    )
    val letters: CategoryModel = CategoryModel(id = CategoryIdModel(value = "letters"), name = "Letters")

    /** A catalog of [documents] with the categories above, every tag they use, and [settings]. */
    fun catalog(
        vararg documents: DocumentModel,
        settings: CatalogSettingsModel = CatalogSettingsModel(),
        extraTags: Set<TagNameModel> = emptySet(),
    ): CatalogModel {
        return CatalogModel(
            libraryId = LibraryIdModel(value = "library"),
            revision = CatalogRevisionModel.INITIAL,
            documents = documents.associateBy { it.id },
            categories = listOf(invoices, energy, letters).associateBy { it.id },
            tags = documents.flatMap { it.tags }.toSet() + extraTags,
            settings = settings,
        )
    }

    @Suppress("LongParameterList") // One argument per property a test may need to set.
    fun document(
        id: String,
        path: String,
        addedAt: Instant = Instant.parse("2024-01-15T00:00:00Z"),
        documentDate: LocalDate? = null,
        title: String? = null,
        correspondent: String? = null,
        categoryId: CategoryIdModel? = null,
        tags: Set<TagNameModel> = emptySet(),
        fields: Map<FieldKeyModel, FieldValue> = emptyMap(),
    ): DocumentModel {
        return DocumentModel(
            id = DocumentIdModel(value = id),
            path = RelativePathModel.of(raw = path),
            contentHash = ContentHashModel(value = id.padStart(length = 64, padChar = '0')),
            size = 1,
            addedAt = addedAt,
            metadata = DocumentMetadataModel(title = title, documentDate = documentDate, correspondent = correspondent),
            categoryId = categoryId,
            tags = tags,
            fields = fields,
        )
    }
}
