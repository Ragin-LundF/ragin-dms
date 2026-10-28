package dms.ai.rebuild

import dms.ai.AiAdapters
import dms.application.pdf.rebuild.LineRepair
import dms.application.port.ai.model.PageImageModel
import dms.application.port.ai.model.layout.LayoutOutcome
import dms.application.port.ai.model.layout.LineOutcome
import dms.application.port.state.model.AiSettingsModel
import dms.domain.rebuild.model.LayoutBoxModel
import dms.domain.rebuild.model.LayoutLineModel
import kotlinx.coroutines.test.runTest
import org.junit.jupiter.api.Assumptions.assumeTrue
import java.awt.Color
import java.awt.Font
import java.awt.RenderingHints
import java.awt.image.BufferedImage
import java.io.ByteArrayOutputStream
import javax.imageio.ImageIO
import kotlin.test.Test
import kotlin.test.assertIs
import kotlin.test.assertTrue
import kotlin.time.Duration.Companion.minutes

/**
 * Sends a synthetic invoice page to the local model the user runs. Opt-in only:
 * `./gradlew :dms-ai:test -Pai.integration=true`, optionally with `-Pai.baseUrl` and `-Pai.model`.
 */
class LayoutReaderIntegrationTest {

    private val settings = AiSettingsModel(
        baseUrl = System.getProperty("ai.baseUrl") ?: AiSettingsModel.DEFAULT_BASE_URL,
        model = System.getProperty("ai.model"),
        timeoutSeconds = 900,
    )
    private val reader = AiAdapters(settings = { settings }, locate = { _, _ -> null }).layoutReader

    @Test
    fun `the local model reads most lines of a synthetic invoice with their umlauts`() {
        assumeTrue(System.getProperty("ai.integration") == "true", "Run with -Pai.integration=true")
        runTest(timeout = 20.minutes) {
            val page = invoice()

            val outcome = reader.readLayout(page = png(image = page))

            val lines = assertIs<LayoutOutcome.Read>(value = outcome).layout.lines
                .map { repaired(line = it, page = page) }
                .map { it.trim().replace(regex = Regex(pattern = "\\s+"), replacement = " ") }
            val found = LINES.count { it in lines }
            assertTrue(actual = found >= LINES.size * 8 / 10, message = "Found $found of ${LINES.size}: $lines")
            assertTrue(actual = LINES.filter { it.any { c -> c in "äöüßÄÖÜ" } }.all { it in lines }, message = "$lines")
        }
    }

    /** A line with lost characters, read again from its crop as the rebuild does. */
    private suspend fun repaired(line: LayoutLineModel, page: BufferedImage): String {
        if (!LineRepair.hasLostCharacters(text = line.text)) return line.text
        val reread = reader.readLine(line = png(image = cropOf(page = page, box = line.box))) as? LineOutcome.Read
        return reread?.let { LineRepair.repaired(text = line.text, reread = it.text) } ?: line.text
    }

    private fun cropOf(page: BufferedImage, box: LayoutBoxModel): BufferedImage {
        val x = pixels(value = box.left, size = page.width).coerceIn(minimumValue = 0, maximumValue = page.width - 1)
        val y = pixels(value = box.top, size = page.height).coerceIn(minimumValue = 0, maximumValue = page.height - 1)
        val right = pixels(value = box.right, size = page.width)
            .coerceIn(minimumValue = x + 1, maximumValue = page.width)
        val bottom = pixels(value = box.bottom, size = page.height)
            .coerceIn(minimumValue = y + 1, maximumValue = page.height)
        return page.getSubimage(x, y, right - x, bottom - y)
    }

    private fun pixels(value: Double, size: Int): Int {
        return (value * size / LayoutBoxModel.SCALE).toInt()
    }

    /** An A4 page at 200 dpi with synthetic invoice lines. */
    private fun invoice(): BufferedImage {
        val page = BufferedImage(1654, 2339, BufferedImage.TYPE_INT_RGB)
        val graphics = page.createGraphics()
        graphics.color = Color.WHITE
        graphics.fillRect(0, 0, page.width, page.height)
        graphics.setRenderingHint(RenderingHints.KEY_TEXT_ANTIALIASING, RenderingHints.VALUE_TEXT_ANTIALIAS_ON)
        graphics.color = Color.BLACK
        graphics.font = Font(Font.SANS_SERIF, Font.PLAIN, 34)
        LINES.forEachIndexed { i, text -> graphics.drawString(text, 160, 240 + i * 90) }
        graphics.dispose()
        return page
    }

    private fun png(image: BufferedImage): PageImageModel {
        val output = ByteArrayOutputStream()
        ImageIO.write(image, "png", output)
        return PageImageModel(bytes = output.toByteArray(), mimeType = "image/png")
    }

    private companion object {
        val LINES = listOf(
            "Beispiel Handwerk GmbH",
            "Musterstraße 12, 12345 Beispielstadt",
            "Rechnung Nr. 2026-0815",
            "Rechnungsdatum: 14.03.2026",
            "Herrn Jürgen Beispiel",
            "Gartenweg 7, 54321 Übungsdorf",
            "Pos. 1 Wartung der Heizungsanlage 240,00 €",
            "Pos. 2 Prüfung der Abgaswerte 85,50 €",
            "Zwischensumme 325,50 €",
            "Umsatzsteuer 19 % 61,85 €",
            "Gesamtbetrag 387,35 €",
            "Zahlbar innerhalb von 14 Tagen ohne Abzüge.",
        )
    }
}
