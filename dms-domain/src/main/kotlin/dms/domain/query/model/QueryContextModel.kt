package dms.domain.query.model

import dms.domain.catalog.model.CatalogModel
import java.time.LocalDate
import java.time.ZoneId

/**
 * What a [DocumentQueryModel] is evaluated against: the catalog (for the category tree), today (for
 * relative conditions such as "due within 30 days") and the zone that turns "added at" into a day.
 */
data class QueryContextModel(
    val catalog: CatalogModel,
    val today: LocalDate,
    val zone: ZoneId,
)
