package dms.pdf.rebuild.fit

import dms.domain.rebuild.KeepReason
import dms.domain.rebuild.model.LayoutLineModel
import dms.domain.rebuild.model.PageLayoutModel
import dms.pdf.rebuild.PixelBoxes
import dms.pdf.rebuild.fit.model.FittedLine
import dms.pdf.rebuild.fit.model.TextRunModel
import dms.pdf.rebuild.font.FontMetrics
import dms.pdf.rebuild.font.FontSet
import dms.pdf.rebuild.font.model.FontStyleModel
import dms.pdf.rebuild.image.Mask
import dms.pdf.rebuild.image.PageMasks
import java.awt.Rectangle

/**
 * Places the AI's lines on the page, measured on the ink rather than trusted from the AI: the box is
 * moved onto the glyph ink, the baseline is measured, words are matched to ink gaps, the size comes from
 * the ink width, and sizes are smoothed across the page. A line stays an image if the AI is not sure, no
 * ink is there, a stamp or handwriting covers it, the font cannot encode it, or the size from the width
 * disagrees with the height of the ink (missing or invented words).
 */
class LineFitter(
    private val masks: PageMasks,
    private val fonts: FontSet,
    private val glyph: Mask = masks.glyphInk,
) {

    /** For print that scans lighter than the ink threshold (grey or coloured text). */
    private val faint by lazy { LineFitter(masks = masks, fonts = fonts, glyph = masks.faintGlyphInk) }

    fun fit(layout: PageLayoutModel, boxes: PixelBoxes): List<FittedLine> {
        val covers = layout.graphics
            .filter { graphic -> COVERING.any { graphic.kind.contains(other = it, ignoreCase = true) } }
            .map { boxes.toPixels(box = it.box) }
        val fitted = layout.lines
            .map { line -> fitWithRetry(line = line, box = boxes.toPixels(box = line.box), covers = covers) }
            .sortedWith(compareBy({ it.box.y / READING_ROW }, { it.box.x }))
        return SizeSmoothing.smooth(lines = fitted, fonts = fonts)
    }

    /** Grey or coloured print is only partly darker than the ink threshold: it is measured on the light mask too. */
    private fun fitWithRetry(line: LayoutLineModel, box: Rectangle, covers: List<Rectangle>): FittedLine {
        val first = fitOne(line = line, predicted = box, covers = covers)
        val retry = first is FittedLine.Kept &&
            (first.reason == KeepReason.NO_INK || first.reason == KeepReason.TEXT_MISMATCH)
        if (!retry || glyph !== masks.glyphInk) return first
        val second = faint.fitOne(line = line, predicted = box, covers = covers)
        val secondFoundInk = second !is FittedLine.Kept || second.reason != KeepReason.NO_INK
        return when {
            second is FittedLine.Text -> second
            first is FittedLine.Kept && first.reason == KeepReason.NO_INK && secondFoundInk -> second
            else -> first
        }
    }

    private fun fitOne(line: LayoutLineModel, predicted: Rectangle, covers: List<Rectangle>): FittedLine {
        val text = line.text.trim().replace(regex = WHITESPACE, replacement = " ")
        val known = line.kept ?: KeepReason.NOT_SURE.takeIf { text.isEmpty() }
        if (known != null) return FittedLine.Kept(box = predicted, reason = known)
        val box = Snap.snap(glyph = glyph, predicted = predicted)
            ?: return FittedLine.Kept(box = predicted, reason = KeepReason.NO_INK)
        val style = FontStyleModel(family = line.family, bold = line.bold)
        val reason = when {
            !line.legible -> KeepReason.NOT_SURE
            covers.any { overlap(cover = it, box = box) > COVER_SHARE } -> KeepReason.COVERED
            !FontMetrics.canEncode(font = fonts.of(style = style), text = text) -> KeepReason.NOT_ENCODABLE
            else -> return measure(text = text, box = box, style = style)
        }
        return FittedLine.Kept(box = box, reason = reason)
    }

    private fun measure(text: String, box: Rectangle, style: FontStyleModel): FittedLine {
        val font = fonts.of(style = style)
        val baseline = Baseline.of(glyph = glyph, box = box)
        val words = text.split(' ')
        val runs = WordInk.of(glyph = glyph, box = box, baseline = baseline, count = words.size)
            ?.let { inks -> words.zip(other = inks) { word, ink -> TextRunModel(text = word, ink = ink) } }
            ?: listOf(TextRunModel(text = text, ink = WordInk.tighten(glyph = glyph, box = box, baseline = baseline)))
        val inkWidth = runs.sumOf { it.ink.width.toDouble() }.toFloat()
        val size = if (text.length < MIN_MEASURED) {
            box.height * SHORT_SIZE
        } else {
            inkWidth / runs.sumOf { FontMetrics.widthAt1(font = font, text = it.text).toDouble() }.toFloat()
        }
        val sizeFromHeight = (baseline - box.y) / FontMetrics.inkTop(font = font, text = text)
        if (text.length >= MIN_MEASURED && size / sizeFromHeight !in MIN_AGREEMENT..MAX_AGREEMENT) {
            return FittedLine.Kept(box = box, reason = KeepReason.TEXT_MISMATCH)
        }
        return FittedLine.Text(
            box = box,
            style = style,
            runs = runs,
            baseline = baseline,
            sizePx = size,
            color = InkColor.of(masks = masks, glyph = glyph, box = box),
        )
    }

    /** The share of [box] that [cover] hides. */
    private fun overlap(cover: Rectangle, box: Rectangle): Double {
        val common = cover.intersection(box)
        if (common.isEmpty) return 0.0
        return common.width.toDouble() * common.height / (box.width.toDouble() * box.height)
    }

    private companion object {
        const val MIN_AGREEMENT = 0.75f
        const val MAX_AGREEMENT = 1.33f
        const val COVER_SHARE = 0.2
        const val MIN_MEASURED = 3
        const val SHORT_SIZE = 0.9f
        const val READING_ROW = 20
        val WHITESPACE = Regex(pattern = "\\s+")
        val COVERING = listOf("stamp", "signature", "handwrit", "seal")
    }
}
