package dms.pdf.rebuild.font

import dms.domain.rebuild.FontFamily
import dms.pdf.rebuild.font.model.FontStyleModel
import org.apache.pdfbox.pdmodel.PDDocument
import org.apache.pdfbox.pdmodel.font.PDType0Font

/**
 * The bundled Liberation fonts (SIL OFL 1.1), loaded into one document as embedded subsets. Their metrics
 * match Arial, Times and Courier, so a rebuilt page looks the same on every system and umlauts, ß and €
 * work. Measuring a text does not add it to a subset; only showing it does.
 */
class FontSet(document: PDDocument) {

    private val fonts: Map<FontStyleModel, PDType0Font> = FILES.mapValues { (_, file) ->
        val stream = checkNotNull(value = FontSet::class.java.getResourceAsStream("/fonts/$file")) {
            "Missing font resource $file"
        }
        stream.use { PDType0Font.load(document, it, true) }
    }

    fun of(style: FontStyleModel): PDType0Font {
        return fonts.getValue(key = style)
    }

    private companion object {
        val FILES = mapOf(
            FontStyleModel(family = FontFamily.SANS, bold = false) to "LiberationSans-Regular.ttf",
            FontStyleModel(family = FontFamily.SANS, bold = true) to "LiberationSans-Bold.ttf",
            FontStyleModel(family = FontFamily.SERIF, bold = false) to "LiberationSerif-Regular.ttf",
            FontStyleModel(family = FontFamily.SERIF, bold = true) to "LiberationSerif-Bold.ttf",
            FontStyleModel(family = FontFamily.MONO, bold = false) to "LiberationMono-Regular.ttf",
            FontStyleModel(family = FontFamily.MONO, bold = true) to "LiberationMono-Bold.ttf",
        )
    }
}
