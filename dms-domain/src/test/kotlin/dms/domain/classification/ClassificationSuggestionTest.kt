package dms.domain.classification

import dms.domain.document.DocumentId
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class ClassificationSuggestionTest {

    private val documentId = DocumentId(value = "d1")

    @Test
    fun `a suggested file name must satisfy the file name policy`() {
        assertFailsWith<IllegalArgumentException> {
            ClassificationSuggestion(documentId = documentId, model = "local-model", fileName = "a:b.pdf")
        }
        val valid = ClassificationSuggestion(documentId = documentId, model = "local-model", fileName = "a b.pdf")
        assertEquals(expected = "a b.pdf", actual = valid.fileName)
    }

    @Test
    fun `confidence must be between zero and one`() {
        assertFailsWith<IllegalArgumentException> {
            ClassificationSuggestion(documentId = documentId, model = "local-model", confidence = 1.5)
        }
    }
}
