package dms.domain.document

import dms.domain.library.model.RelativePathModel
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse

class DocumentKindTest {

    @Test
    fun `kind is decided by the extension regardless of case`() {
        val expectations = mapOf(
            "a/scan.PDF" to DocumentKind.PDF,
            "photo.jpeg" to DocumentKind.IMAGE,
            "photo.png" to DocumentKind.IMAGE,
            "letter.docx" to DocumentKind.OFFICE,
            "slides.pptx" to DocumentKind.OFFICE,
            "sheet.xlsx" to DocumentKind.OFFICE,
            "audio.mp3" to DocumentKind.OTHER,
            "no-extension" to DocumentKind.OTHER,
        )

        expectations.forEach { (path, kind) ->
            assertEquals(
                expected = kind,
                actual = DocumentKind.of(path = RelativePathModel.of(raw = path)),
                message = path,
            )
        }
    }

    @Test
    fun `other files are not readable by the AI`() {
        assertFalse(actual = DocumentKind.OTHER.readableByAi)
    }
}
