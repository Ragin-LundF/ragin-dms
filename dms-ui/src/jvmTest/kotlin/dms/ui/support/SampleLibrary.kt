package dms.ui.support

import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import kotlinx.coroutines.flow.first

/**
 * A [TestApp] library whose primary and sync folder hold the same synthetic files, catalogued while it
 * opens, and the catalog as it is now. Only invented file names and contents.
 */
object SampleLibrary {

    /**
     * Writes [paths] with distinct contents into both folders, opens the library and returns the
     * catalogued documents by path.
     */
    suspend fun open(app: TestApp, vararg paths: String): Map<String, DocumentModel> {
        paths.forEachIndexed { index, path ->
            app.write(file = app.primary.resolve(path), content = "synthetic content $index")
            app.write(file = app.sync.resolve(path), content = "synthetic content $index")
        }
        app.configure(addNewFiles = true)
        val catalog = app.services.library.catalog.first { it != null && it.documents.size == paths.size }
        return checkNotNull(value = catalog).documents.values.associateBy { it.path.value }
    }

    fun catalog(app: TestApp): CatalogModel {
        return checkNotNull(value = app.services.library.catalog.value)
    }

    fun document(app: TestApp, id: DocumentIdModel): DocumentModel {
        return checkNotNull(value = catalog(app = app).documents[id])
    }
}
