package dms.storage.catalog.mapper

import dms.domain.field.model.FieldKeyModel
import dms.domain.library.model.RelativePathModel
import dms.domain.query.FieldComparison
import dms.domain.query.model.DateRangeModel
import dms.domain.query.model.DocumentQueryModel
import dms.domain.query.model.FieldConditionModel
import dms.domain.query.model.SavedSearchIdModel
import dms.domain.query.model.SavedSearchModel
import dms.domain.query.model.TagFilterModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.TagNameModel
import dms.storage.catalog.dto.settings.FieldConditionDto
import dms.storage.catalog.dto.settings.QueryDto
import dms.storage.catalog.dto.settings.SavedSearchDto
import java.time.Instant
import java.time.LocalDate

/** Maps saved searches and their queries. Only the conditions a query sets are written. */
object QueryMapper {

    fun toDto(search: SavedSearchModel): SavedSearchDto {
        return SavedSearchDto(id = search.id.value, name = search.name, query = toDto(query = search.query))
    }

    fun toDomain(search: SavedSearchDto): SavedSearchModel {
        return SavedSearchModel(
            id = SavedSearchIdModel(value = search.id),
            name = search.name,
            query = toDomain(query = search.query),
        )
    }

    fun toDto(query: DocumentQueryModel): QueryDto {
        return QueryDto(
            text = query.text,
            category = query.category?.value,
            uncategorized = query.uncategorized.takeIf { it },
            tagsAll = query.tags.all.sorted().map { it.value },
            tagsAny = query.tags.any.sorted().map { it.value },
            tagsNone = query.tags.none.sorted().map { it.value },
            untagged = query.untagged.takeIf { it },
            from = query.dates?.from?.toString(),
            to = query.dates?.to?.toString(),
            correspondent = query.correspondent,
            folder = query.folder?.value,
            fields = query.fields.map { condition ->
                FieldConditionDto(
                    key = condition.key.value,
                    comparison = condition.comparison.name,
                    value = condition.value?.let { FieldMapper.toDto(value = it) },
                )
            },
            addedSince = query.addedSince?.toString(),
            contentText = query.contentText,
        )
    }

    fun toDomain(query: QueryDto): DocumentQueryModel {
        val from = query.from?.let { LocalDate.parse(it) }
        val to = query.to?.let { LocalDate.parse(it) }
        return DocumentQueryModel(
            text = query.text,
            category = query.category?.let { CategoryIdModel(value = it) },
            uncategorized = query.uncategorized == true,
            tags = TagFilterModel(
                all = tagsOf(names = query.tagsAll),
                any = tagsOf(names = query.tagsAny),
                none = tagsOf(names = query.tagsNone),
            ),
            untagged = query.untagged == true,
            dates = if (from == null && to == null) null else DateRangeModel(from = from, to = to),
            correspondent = query.correspondent,
            folder = query.folder?.let { RelativePathModel.of(raw = it) },
            fields = query.fields.map { condition ->
                FieldConditionModel(
                    key = FieldKeyModel(value = condition.key),
                    comparison = FieldComparison.valueOf(condition.comparison),
                    value = condition.value?.let { FieldMapper.toDomain(value = it) },
                )
            },
            addedSince = query.addedSince?.let { Instant.parse(it) },
            contentText = query.contentText,
        )
    }

    private fun tagsOf(names: List<String>): Set<TagNameModel> {
        return names.map { TagNameModel.of(raw = it) }.toSet()
    }
}
