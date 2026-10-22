package dms.domain.deadline

import dms.domain.catalog.model.CatalogModel
import dms.domain.deadline.model.UpcomingDateModel
import dms.domain.field.FieldRole
import dms.domain.field.FieldValue
import dms.domain.retention.RetentionDates
import dms.domain.task.OpenTask
import java.time.LocalDate

/**
 * Turns the deadline fields and the opt-in retention of the documents into dashboard tasks and the
 * "Upcoming" list. A deadline is announced its role's lead time ahead and stays for as long again
 * after it passed; a retention task appears once its date is reached.
 */
object DeadlinePlanner {

    /** The deadline and retention tasks due on [today]. */
    fun dueTasks(
        catalog: CatalogModel,
        today: LocalDate,
        leadDays: (FieldRole) -> Int = { it.defaultLeadDays },
    ): List<OpenTask> {
        val deadlines = deadlineDates(catalog = catalog).filter { (role, upcoming) ->
            val lead = leadDays(role).toLong()
            !upcoming.date.isAfter(today.plusDays(lead)) && !upcoming.date.isBefore(today.minusDays(lead))
        }.map { (_, upcoming) ->
            OpenTask.DeadlineDue(
                documentId = upcoming.documentId,
                fieldKey = checkNotNull(value = upcoming.field),
                date = upcoming.date,
            )
        }
        val retention = retentionDates(catalog = catalog)
            .filter { !it.date.isAfter(today) }
            .map { OpenTask.RetentionDue(documentId = it.documentId, date = it.date) }
        return deadlines + retention
    }

    /** The next [limit] deadline and retention dates from [today] on, earliest first. */
    fun upcoming(catalog: CatalogModel, today: LocalDate, limit: Int): List<UpcomingDateModel> {
        val dates = deadlineDates(catalog = catalog).map { it.second } + retentionDates(catalog = catalog)
        return dates.filter { !it.date.isBefore(today) }
            .sortedWith(compareBy({ it.date }, { it.documentId.value }))
            .take(n = limit)
    }

    private fun deadlineDates(catalog: CatalogModel): List<Pair<FieldRole, UpcomingDateModel>> {
        val roles = catalog.settings.fieldDefinitions.mapNotNull { definition ->
            definition.role?.let { definition.key to it }
        }.toMap()
        return catalog.documents.values.flatMap { document ->
            document.fields.mapNotNull { (key, value) ->
                val role = roles[key] ?: return@mapNotNull null
                val date = (value as? FieldValue.Date)?.date ?: return@mapNotNull null
                role to UpcomingDateModel(documentId = document.id, field = key, date = date)
            }
        }
    }

    private fun retentionDates(catalog: CatalogModel): List<UpcomingDateModel> {
        return catalog.documents.values.mapNotNull { document ->
            RetentionDates.dueDate(document = document, catalog = catalog)?.let {
                UpcomingDateModel(documentId = document.id, field = null, date = it)
            }
        }
    }
}
