package dms.storage.catalog.mapper

import dms.domain.catalog.model.CatalogModel
import dms.domain.catalog.model.CatalogRevisionModel
import dms.domain.catalog.model.LibraryIdModel
import dms.domain.document.ClassificationSource
import dms.domain.document.model.ClassificationRecordModel
import dms.domain.document.model.ContentHashModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.link.model.DocumentLinkModel
import dms.domain.retention.model.RetentionModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import dms.domain.taxonomy.model.TagNameModel
import dms.storage.catalog.CatalogMigrations
import dms.storage.catalog.dto.CatalogDto
import dms.storage.catalog.dto.CategoryDto
import dms.storage.catalog.dto.ClassificationDto
import dms.storage.catalog.dto.DocumentDto
import dms.storage.catalog.dto.LinkDto
import dms.storage.catalog.dto.RetentionDto
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
    fun toDto(catalog: CatalogModel, previous: CatalogDto?): CatalogDto {
        val categories = catalog.categories.values
            .sortedWith(compareBy({ it.name.lowercase(Locale.ROOT) }, { it.id.value }))
            .map { toDto(category = it) }
        val previousDocuments = previous?.documents.orEmpty().associateBy { it.id }
        val documents = catalog.documents.values
            .sortedBy { it.path.value }
            .map { document ->
                val previousDocument = previousDocuments[document.id.value]
                val mapped = toDto(document = document, previousRetention = previousDocument?.retention)
                UnknownFields.carry(entry = mapped, previous = previousDocument)
                mapped
            }
        val links = catalog.links
            .sortedWith(compareBy({ it.first.value }, { it.second.value }))
            .map { LinkDto(first = it.first.value, second = it.second.value, label = it.label) }
        val base = CatalogDto(
            formatVersion = CatalogMigrations.CURRENT_VERSION,
            libraryId = catalog.libraryId.value,
            revision = catalog.revision.value,
            categories = UnknownFields.carry(entries = categories, previous = previous?.categories.orEmpty()) { it.id },
            tags = sortedTags(tags = catalog.tags),
            documents = documents,
            links = UnknownFields.carry(
                entries = links,
                previous = previous?.links.orEmpty(),
            ) { it.first to it.second },
        )
        // copy() does not carry the unknown fields, so they are added last.
        val dto = SettingsMapper.withSettings(dto = base, settings = catalog.settings, previous = previous)
        UnknownFields.carry(entry = dto, previous = previous)
        return dto
    }

    /** The catalog in [dto]. Throws [IllegalArgumentException] if the file breaks a catalog rule. */
    fun toDomain(dto: CatalogDto): CatalogModel {
        val documents = dto.documents.map { toDomain(document = it) }
        return CatalogModel(
            libraryId = LibraryIdModel(value = dto.libraryId),
            revision = CatalogRevisionModel(value = dto.revision),
            documents = documents.associateBy { it.id },
            categories = dto.categories.map { toDomain(category = it) }.associateBy { it.id },
            tags = dto.tags.map { TagNameModel.of(raw = it) }.toSet() + documents.flatMap { it.tags },
            links = linksOf(dto = dto, documents = documents.map { it.id }.toSet()),
            settings = SettingsMapper.toDomain(dto = dto),
        )
    }

    /**
     * The links between documents the file has. An older version that removed a document keeps the
     * links it did not understand, so links to missing documents are dropped here instead of making
     * the whole file unreadable.
     */
    private fun linksOf(dto: CatalogDto, documents: Set<DocumentIdModel>): Set<DocumentLinkModel> {
        return dto.links
            .filter { link ->
                DocumentIdModel(value = link.first) in documents && DocumentIdModel(value = link.second) in documents
            }
            .filter { it.first != it.second }
            .map { link ->
                DocumentLinkModel.of(
                    first = DocumentIdModel(value = link.first),
                    second = DocumentIdModel(value = link.second),
                    label = link.label?.takeIf { it.isNotBlank() },
                )
            }
            .distinctBy { it.first to it.second }
            .toSet()
    }

    private fun toDto(category: CategoryModel): CategoryDto {
        return CategoryDto(
            id = category.id.value,
            name = category.name,
            parentId = category.parentId?.value,
            retentionYears = category.retentionYears,
        )
    }

    private fun toDto(document: DocumentModel, previousRetention: RetentionDto?): DocumentDto {
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
            fields = FieldMapper.toDto(fields = document.fields),
            note = document.note,
            retention = document.retention?.let {
                val retention = RetentionDto(keepUntil = it.keepUntil?.toString(), settledAt = it.settledAt?.toString())
                UnknownFields.carry(entry = retention, previous = previousRetention)
            },
        )
    }

    private fun toDomain(category: CategoryDto): CategoryModel {
        return CategoryModel(
            id = CategoryIdModel(value = category.id),
            name = category.name,
            parentId = category.parentId?.let { CategoryIdModel(value = it) },
            retentionYears = category.retentionYears,
        )
    }

    private fun toDomain(document: DocumentDto): DocumentModel {
        return DocumentModel(
            id = DocumentIdModel(value = document.id),
            path = RelativePathModel.of(raw = document.path),
            contentHash = ContentHashModel.fromHex(hex = document.contentHash),
            size = document.size,
            addedAt = Instant.parse(document.addedAt),
            metadata = DocumentMetadataModel(
                title = document.title,
                documentDate = document.documentDate?.let { LocalDate.parse(it) },
                correspondent = document.correspondent,
            ),
            categoryId = document.categoryId?.let { CategoryIdModel(value = it) },
            tags = document.tags.map { TagNameModel.of(raw = it) }.toSet(),
            classification = document.classification?.let { record ->
                ClassificationRecordModel(
                    source = ClassificationSource.valueOf(record.source),
                    model = record.model,
                    confirmedAt = Instant.parse(record.confirmedAt),
                )
            },
            fields = FieldMapper.toDomain(fields = document.fields),
            note = document.note?.takeIf { it.isNotBlank() },
            retention = document.retention?.let { retention ->
                RetentionModel(
                    keepUntil = retention.keepUntil?.let { LocalDate.parse(it) },
                    settledAt = retention.settledAt?.let { Instant.parse(it) },
                ).takeUnless { it.isEmpty }
            },
        )
    }

    private fun sortedTags(tags: Set<TagNameModel>): List<String> {
        return tags.sorted().map { it.value }
    }
}
