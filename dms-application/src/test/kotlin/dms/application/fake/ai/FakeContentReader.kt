package dms.application.fake.ai

import dms.application.port.ai.DocumentContentReader
import dms.application.port.ai.model.DocumentContent
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.LibraryFolderModel

/** Reads every document as [content]; [texts] are the text layers by file name. */
class FakeContentReader : DocumentContentReader {

    var content: DocumentContent = DocumentContent.Text(text = "synthetic text")
    var texts: Map<String, String> = emptyMap()
    var textReads: Int = 0
        private set

    override suspend fun read(folder: LibraryFolderModel, document: DocumentModel): DocumentContent {
        return content
    }

    override suspend fun text(folder: LibraryFolderModel, document: DocumentModel): String? {
        textReads++
        return texts[document.path.fileName]
    }
}
