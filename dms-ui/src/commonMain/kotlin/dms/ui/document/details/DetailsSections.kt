package dms.ui.document.details

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Rule
import androidx.compose.runtime.Composable
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentModel
import dms.domain.retention.RetentionDates
import dms.ui.components.ActionButtons
import dms.ui.components.SectionHeader
import dms.ui.components.model.ActionButtonModel

/** The quickLookViewModel's sections for fields, a rule from the document, and retention. */
@Composable
fun DetailsSections(
    document: DocumentModel,
    catalog: CatalogModel,
    busyReason: String?,
    controller: DetailsController,
) {
    SectionHeader(title = "Fields")
    FieldsEditor(
        document = document,
        definitions = catalog.settings.fieldDefinitions,
        busyReason = busyReason,
        onSave = { controller.saveFields(document = document, typed = it) },
        onAddStandardFields = controller::addStandardFields,
    )
    RuleSection(document = document, busyReason = busyReason, controller = controller)
    SectionHeader(title = "Retention")
    val category = document.categoryId?.let { catalog.categories[it] }
    RetentionEditor(
        document = document,
        category = category,
        dueDate = RetentionDates.dueDate(document = document, catalog = catalog),
        busyReason = busyReason,
        onKeepUntil = { controller.saveKeepUntil(document = document, typed = it) },
        onCategoryYears = { typed ->
            category?.let { controller.saveCategoryYears(categoryId = it.id, typed = typed) }
        },
    )
}

@Composable
private fun RuleSection(document: DocumentModel, busyReason: String?, controller: DetailsController) {
    SectionHeader(title = "Rule")
    ActionButtons(
        buttons = listOf(
            ActionButtonModel(
                label = "Create rule from this document",
                icon = Icons.Rounded.Rule,
                disabledReason = "Give the document a correspondent first"
                    .takeIf { document.metadata.correspondent == null },
            ) { controller.createRule(document = document) },
        ),
        enabled = busyReason == null,
        busyReason = busyReason,
    )
}
