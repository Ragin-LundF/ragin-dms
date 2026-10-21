package dms.application.fake.ai

import dms.application.port.ai.DocumentContentReader
import dms.application.port.ai.model.DocumentContent
import dms.domain.document.Document
import dms.domain.library.LibraryFolder

/** Reads every document as [content]. */
class FakeContentReader : DocumentContentReader {

    var content: DocumentContent = DocumentContent.Text(text = "synthetic text")

    override suspend fun read(folder: LibraryFolder, document: Document): DocumentContent {
        return content
    }
}
