package dms.ui.document.tab

import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentModel
import dms.domain.field.FieldValue
import dms.domain.field.FieldValues
import dms.domain.field.model.FieldDefinitionModel
import dms.domain.retention.RetentionDates
import dms.ui.components.format.DateTexts
import dms.ui.components.format.FileSizes
import dms.ui.components.surface.KindStyles
import dms.ui.document.model.DocumentFactModel
import java.time.LocalDate
import java.time.ZoneId
import java.time.temporal.ChronoUnit

/**
 * The details of a document as label and value: category, tags, correspondent, date, the custom
 * fields it has values for, folder, size and kind, when it was added, and its retention. A deadline
 * field gets a chip with the time left.
 */
object DocumentFacts {

    fun of(document: DocumentModel, catalog: CatalogModel, today: LocalDate, zone: ZoneId): List<DocumentFactModel> {
        val metadata = document.metadata
        return buildList {
            add(fact(label = "Category", value = document.categoryId?.let { catalog.categories[it]?.name }))
            add(fact(label = "Tags", value = document.tags.sorted().joinToString(separator = ", ") { it.value }))
            metadata.correspondent?.let { add(fact(label = "Correspondent", value = it)) }
            metadata.documentDate?.let { add(fact(label = "Document date", value = DateTexts.absolute(date = it))) }
            catalog.settings.fieldDefinitions.forEach { definition ->
                document.fields[definition.key]?.let { value ->
                    add(fieldFact(definition = definition, value = value, today = today))
                }
            }
            add(fact(label = "Folder", value = document.path.parent?.value?.ifEmpty { null } ?: "Top level"))
            add(
                fact(
                    label = "Size",
                    value = "${FileSizes.format(bytes = document.size)} · ${KindStyles.label(kind = document.kind)}",
                ),
            )
            add(fact(label = "Added", value = DateTexts.absolute(instant = document.addedAt, zone = zone)))
            add(fact(label = "Retention", value = retentionOf(document = document, catalog = catalog)))
        }
    }

    private fun fieldFact(definition: FieldDefinitionModel, value: FieldValue, today: LocalDate): DocumentFactModel {
        val text = FieldValues.format(value = value)
        val role = definition.role
        if (value !is FieldValue.Date || role == null) {
            return DocumentFactModel(label = definition.label, value = text)
        }
        return DocumentFactModel(
            label = definition.label,
            value = text,
            chip = DateTexts.relative(date = value.date, today = today),
            urgent = ChronoUnit.DAYS.between(today, value.date) <= role.defaultLeadDays,
        )
    }

    private fun retentionOf(document: DocumentModel, catalog: CatalogModel): String {
        if (document.retention?.settledAt != null) {
            return "Done"
        }
        val due = RetentionDates.dueDate(document = document, catalog = catalog) ?: return "Not set"
        return "Until ${DateTexts.absolute(date = due)}"
    }

    private fun fact(label: String, value: String?): DocumentFactModel {
        return DocumentFactModel(label = label, value = value?.ifEmpty { null } ?: "None")
    }
}
