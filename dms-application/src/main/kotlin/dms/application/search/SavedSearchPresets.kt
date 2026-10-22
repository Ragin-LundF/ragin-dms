package dms.application.search

import dms.application.port.system.IdGenerator
import dms.domain.field.FieldPresets
import dms.domain.field.FieldValue
import dms.domain.query.FieldComparison
import dms.domain.query.model.DateRangeModel
import dms.domain.query.model.DocumentQueryModel
import dms.domain.query.model.FieldConditionModel
import dms.domain.query.model.SavedSearchIdModel
import dms.domain.query.model.SavedSearchModel
import dms.domain.query.model.TagFilterModel
import dms.domain.taxonomy.model.TagNameModel
import java.math.BigDecimal
import java.time.LocalDate

/**
 * The smart folders a library starts with: "Untagged", "Uncategorized", the tax documents of the
 * current year, and what is due in the next 30 days.
 */
object SavedSearchPresets {

    private const val DUE_DAYS = 30L

    fun of(today: LocalDate, ids: IdGenerator, taxTag: TagNameModel): List<SavedSearchModel> {
        val presets = listOf(
            "Untagged" to DocumentQueryModel(untagged = true),
            "Uncategorized" to DocumentQueryModel(uncategorized = true),
            "Tax ${today.year}" to DocumentQueryModel(
                tags = TagFilterModel(all = setOf(taxTag)),
                dates = DateRangeModel.year(year = today.year),
            ),
            "Due in 30 days" to DocumentQueryModel(
                fields = listOf(
                    FieldConditionModel(
                        key = FieldPresets.DUE_DATE.key,
                        comparison = FieldComparison.WITHIN_DAYS,
                        value = FieldValue.Number(number = BigDecimal.valueOf(DUE_DAYS)),
                    ),
                ),
            ),
        )
        return presets.map { (name, query) ->
            SavedSearchModel(id = SavedSearchIdModel(value = ids.newId()), name = name, query = query)
        }
    }
}
