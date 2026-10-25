package dms.ui.document.quicklook

import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentModel
import dms.domain.field.FieldRole
import dms.domain.field.FieldType
import dms.domain.field.FieldValue
import dms.domain.field.FieldValues
import dms.domain.field.model.FieldDefinitionModel
import dms.ui.components.format.DateTexts
import dms.ui.document.model.DocumentFactModel
import dms.ui.document.model.QuickFactsModel
import java.time.LocalDate
import java.time.temporal.ChronoUnit

/**
 * The key facts of a document: category, tags, correspondent, date, and the amount and due date
 * when the library has such fields. The amount is the first money field, the due date the first
 * field with the due-date role; every other field stays in the folded details.
 */
object QuickFacts {

    fun of(document: DocumentModel, catalog: CatalogModel, today: LocalDate): QuickFactsModel {
        val metadata = document.metadata
        val fact = { field: FieldDefinitionModel -> factOf(definition = field, document = document, today = today) }
        return QuickFactsModel(
            category = document.categoryId?.let { catalog.categories[it]?.name },
            tags = document.tags.sorted(),
            correspondent = metadata.correspondent,
            documentDate = metadata.documentDate?.let { DateTexts.absolute(date = it) },
            amount = amountField(catalog = catalog)?.let(fact),
            due = dueField(catalog = catalog)?.let(fact),
        )
    }

    fun amountField(catalog: CatalogModel): FieldDefinitionModel? {
        return catalog.settings.fieldDefinitions.firstOrNull { it.type == FieldType.MONEY }
    }

    fun dueField(catalog: CatalogModel): FieldDefinitionModel? {
        return catalog.settings.fieldDefinitions.firstOrNull { it.role == FieldRole.DUE_DATE }
    }

    private fun factOf(definition: FieldDefinitionModel, document: DocumentModel, today: LocalDate): DocumentFactModel {
        val value = document.fields[definition.key]
            ?: return DocumentFactModel(label = definition.label, value = null)
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
}
