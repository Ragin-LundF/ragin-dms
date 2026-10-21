package dms.storage.catalog

import dms.domain.catalog.Catalog
import dms.domain.catalog.CatalogRevision
import dms.domain.catalog.LibraryId
import dms.domain.document.ClassificationRecord
import dms.domain.document.ClassificationSource
import dms.domain.document.ContentHash
import dms.domain.document.Document
import dms.domain.document.DocumentId
import dms.domain.document.DocumentMetadata
import dms.domain.library.RelativePath
import dms.domain.taxonomy.Category
import dms.domain.taxonomy.CategoryId
import dms.domain.taxonomy.TagName
import dms.storage.catalog.dto.CatalogDto
import dms.storage.catalog.dto.CategoryDto
import dms.storage.catalog.dto.ClassificationDto
import dms.storage.catalog.dto.DocumentDto
import java.time.Instant
import java.time.LocalDate
import java.util.Locale

/**
 * Maps between the catalog and its file. The file is sorted (documents by path, categories by name,
 * tags alphabetically) so that saving an unchanged catalog writes the same bytes, and a diff of two
 * versions shows only what changed.
 */
object CatalogMapper {

    /** The file content for [catalog]; fields only known to newer versions are taken over from [previous]. */
    fun toDto(catalog: Catalog, previous: CatalogDto?): CatalogDto {
        val previousDocuments = previous?.documents.orEmpty().associateBy { it.id }
        val previousCategories = previous?.categories.orEmpty().associateBy { it.id }
        val dto = CatalogDto(
            formatVersion = CatalogMigrations.CURRENT_VERSION,
            libraryId = catalog.libraryId.value,
            revision = catalog.revision.value,
            categories = catalog.categories.values
                .sortedWith(compareBy({ it.name.lowercase(Locale.ROOT) }, { it.id.value }))
                .map { category ->
                    val mapped = toDto(category = category)
                    mapped.unknownFields += previousCategories[mapped.id]?.unknownFields.orEmpty()
                    mapped
                },
            tags = sortedTags(tags = catalog.tags),
            documents = catalog.documents.values
                .sortedBy { it.path.value }
                .map { document ->
                    val mapped = toDto(document = document)
                    mapped.unknownFields += previousDocuments[mapped.id]?.unknownFields.orEmpty()
                    mapped
                },
        )
        dto.unknownFields += previous?.unknownFields.orEmpty()
        return dto
    }

    /** The catalog in [dto]. Throws [IllegalArgumentException] if the file breaks a catalog rule. */
    fun toDomain(dto: CatalogDto): Catalog {
        val documents = dto.documents.map { toDomain(document = it) }
        return Catalog(
            libraryId = LibraryId(value = dto.libraryId),
            revision = CatalogRevision(value = dto.revision),
            documents = documents.associateBy { it.id },
            categories = dto.categories.map { toDomain(category = it) }.associateBy { it.id },
            tags = dto.tags.map { TagName.of(raw = it) }.toSet() + documents.flatMap { it.tags },
        )
    }

    private fun toDto(category: Category): CategoryDto {
        return CategoryDto(id = category.id.value, name = category.name, parentId = category.parentId?.value)
    }

    private fun toDto(document: Document): DocumentDto {
        return DocumentDto(
            id = document.id.value,
            path = document.path.value,
            contentHash = document.contentHash.value,
            size = document.size,
            addedAt = document.addedAt.toString(),
            title = document.metadata.title,
            documentDate = document.metadata.documentDate?.toString(),
            correspondent = document.metadata.correspondent,
            categoryId = document.categoryId?.value,
            tags = sortedTags(tags = document.tags),
            classification = document.classification?.let { record ->
                ClassificationDto(
                    source = record.source.name,
                    model = record.model,
                    confirmedAt = record.confirmedAt.toString(),
                )
            },
        )
    }

    private fun toDomain(category: CategoryDto): Category {
        return Category(
            id = CategoryId(value = category.id),
            name = category.name,
            parentId = category.parentId?.let { CategoryId(value = it) },
        )
    }

    private fun toDomain(document: DocumentDto): Document {
        return Document(
            id = DocumentId(value = document.id),
            path = RelativePath.of(raw = document.path),
            contentHash = ContentHash.fromHex(hex = document.contentHash),
            size = document.size,
            addedAt = Instant.parse(document.addedAt),
            metadata = DocumentMetadata(
                title = document.title,
                documentDate = document.documentDate?.let { LocalDate.parse(it) },
                correspondent = document.correspondent,
            ),
            categoryId = document.categoryId?.let { CategoryId(value = it) },
            tags = document.tags.map { TagName.of(raw = it) }.toSet(),
            classification = document.classification?.let { record ->
                ClassificationRecord(
                    source = ClassificationSource.valueOf(record.source),
                    model = record.model,
                    confirmedAt = Instant.parse(record.confirmedAt),
                )
            },
        )
    }

    private fun sortedTags(tags: Set<TagName>): List<String> {
        return tags.sorted().map { it.value }
    }
}
