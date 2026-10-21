package dms.ai.read

import dms.ai.prompt.PromptTexts
import dms.application.port.ai.model.DocumentContent
import org.apache.poi.extractor.ExtractorFactory
import java.nio.file.Files
import java.nio.file.Path

/** The text of a Word, PowerPoint or Excel file. */
object OfficeContent {

    /** The whole text, or an empty string when there is none. */
    fun text(file: Path): String {
        return Files.newInputStream(file).use { input ->
            ExtractorFactory.createExtractor(input).use { it.text }
        }.trim()
    }

    fun read(file: Path): DocumentContent {
        val text = text(file = file)
        return if (text.isEmpty()) {
            DocumentContent.NotReadable(reason = "The file contains no text")
        } else {
            DocumentContent.Text(text = text.take(n = PromptTexts.MAX_TEXT_CHARACTERS))
        }
    }
}
