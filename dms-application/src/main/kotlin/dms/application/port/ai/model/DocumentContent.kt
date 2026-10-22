package dms.application.port.ai.model

/** What a document reader made of a file: text, page images, or nothing usable. */
sealed interface DocumentContent {

    /** Text from a PDF text layer or an Office file. */
    data class Text(val text: String) : DocumentContent

    /** Rendered pages of a scan, or an image file; needs a vision-capable model. */
    class PageImages(val pages: List<PageImageModel>) : DocumentContent

    data class NotReadable(val reason: String) : DocumentContent
}
