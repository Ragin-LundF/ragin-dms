package dms.application.pdf

import dms.application.common.OperationOutcome
import dms.application.library.CatalogStore
import dms.application.port.pdf.PdfEditor
import dms.application.port.pdf.model.PdfMetadataModel
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.DocumentKind
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel

/**
 * Writes the catalog's title, correspondent (author), category (subject), tags (keywords) and date
 * into a PDF, as a new version, so other applications show them. Only while the library's setting
 * for it is on; it is off by default.
 */
class EmbedPdfMetadata(
    private val store: CatalogStore,
    private val pdf: PdfEditor,
    private val versions: PdfVersions,
) {

    suspend operator fun invoke(documentId: DocumentIdModel): OperationOutcome {
        val library = store.current() ?: return OperationOutcome.LibraryNotOpen
        val document = library.catalog.documents[documentId] ?: return OperationOutcome.NotFound
        if (document.kind != DocumentKind.PDF || !library.catalog.settings.layout.embedMetadata) {
            return OperationOutcome.Done(catalog = library.catalog, pendingFolders = emptySet())
        }
        val metadata = metadataOf(document = document, catalog = library.catalog)
        return versions.replace(library = library, document = document) { folder, target ->
            pdf.writeMetadata(folder = folder, source = document.path, metadata = metadata, target = target)
        }
    }

    private fun metadataOf(document: DocumentModel, catalog: CatalogModel): PdfMetadataModel {
        val category = generateSequence(seed = document.categoryId?.let { catalog.categories[it] }) { current ->
            current.parentId?.let { catalog.categories[it] }
        }.take(n = catalog.categories.size).map { it.name }.toList().reversed()
        return PdfMetadataModel(
            title = document.metadata.title,
            author = document.metadata.correspondent,
            subject = category.joinToString(separator = " / ").ifEmpty { null },
            keywords = document.tags.sorted().map { it.value },
            date = document.metadata.documentDate,
        )
    }
}
