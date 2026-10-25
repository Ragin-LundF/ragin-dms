package dms.ui.document.tab

import androidx.compose.runtime.Composable
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.components.SectionHeader
import dms.ui.document.InspectorViewModel
import dms.ui.document.details.DetailsSections
import dms.ui.document.edit.CategoryPicker
import dms.ui.document.edit.MetadataEditor
import dms.ui.document.edit.TagEditor
import dms.ui.library.model.CategoryEntryModel

/** The details at a glance, then the editors for metadata, category, tags, fields, rule and retention. */
@Composable
fun DetailsTab(
    document: DocumentModel,
    catalog: CatalogModel,
    categories: List<CategoryEntryModel>,
    allTags: List<TagNameModel>,
    busyReason: String?,
    viewModel: InspectorViewModel,
) {
    val enabled = busyReason == null
    FactGrid(facts = viewModel.factsOf(document = document, catalog = catalog))
    SectionHeader(title = "Title, date and correspondent")
    MetadataEditor(document = document, enabled = enabled) { draft ->
        viewModel.saveMetadata(document = document, draft = draft)
    }
    SectionHeader(title = "Category")
    CategoryPicker(selected = document.categoryId, categories = categories, enabled = enabled) { categoryId ->
        viewModel.assignCategory(document = document, categoryId = categoryId)
    }
    SectionHeader(title = "Tags")
    TagEditor(
        tags = document.tags.sorted(),
        allTags = allTags,
        enabled = enabled,
        onAdd = { viewModel.addTag(document = document, raw = it) },
        onRemove = { viewModel.removeTag(document = document, tag = it) },
    )
    DetailsSections(document = document, catalog = catalog, busyReason = busyReason, controller = viewModel.details)
}
