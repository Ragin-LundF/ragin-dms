package dms.pdf.rebuild.compose

import dms.pdf.rebuild.compose.model.CropModel
import dms.pdf.rebuild.fit.model.FittedLine
import dms.pdf.rebuild.image.Labels
import dms.pdf.rebuild.image.Mask
import dms.pdf.rebuild.image.PageMasks
import java.awt.Rectangle
import java.awt.image.BufferedImage
import kotlin.math.abs

/**
 * Cuts everything that is not replaced text out of the scan. A component of [PageMasks.light] becomes
 * graphics if less than half of it lies in the text areas; rules always do. Graphics within a few pixels
 * of each other form one crop, which keeps the original pixels on and next to the graphics and white
 * elsewhere, so thin strokes keep their soft edges.
 */
object Crops {

    private const val DUST = 8
    private const val GROUP = 6
    private const val TEXT_PAD = 0.3
    private const val TEXT_SIDE = 2
    private const val COLOUR_SPREAD = 24
    private const val WHITE = 255
    private const val WHITE_RGB = 0xFFFFFF
    private const val BYTE = 0xFF
    private const val RED_SHIFT = 16
    private const val GREEN_SHIFT = 8

    fun of(masks: PageMasks, lines: List<FittedLine>): List<CropModel> {
        val runBoxes = lines.filterIsInstance<FittedLine.Text>().flatMap { line ->
            line.runs.map { Rectangle(it.ink.x, line.box.y, it.ink.width, line.box.height) }
        }
        val keptLines = lines.filterIsInstance<FittedLine.Kept>()
        val graphics = graphicsOf(masks = masks, runBoxes = runBoxes, keptLines = keptLines)
        val keep = graphics.dilate(radius = 1)
        val page = Rectangle(0, 0, masks.width, masks.height)
        val groups = Labels.of(mask = graphics.dilate(radius = GROUP))
        return groups.boxes.map { group ->
            val box = group.intersection(page)
            CropModel(
                box = box,
                image = cut(masks = masks, box = box, keep = keep),
                filled = filledShare(keep = keep, box = box),
            )
        }
    }

    private fun graphicsOf(masks: PageMasks, runBoxes: List<Rectangle>, keptLines: List<FittedLine.Kept>): Mask {
        val textAreas = Mask(width = masks.width, height = masks.height)
        runBoxes.forEach { box ->
            val pad = (box.height * TEXT_PAD).toInt()
            val area = Rectangle(box.x - TEXT_SIDE, box.y - pad, box.width + 2 * TEXT_SIDE, box.height + 2 * pad)
            textAreas.fill(box = area)
        }
        val labels = Labels.of(mask = masks.light)
        val inside = IntArray(size = labels.size)
        labels.ids.forEachIndexed { i, id -> if (id >= 0 && textAreas.bits[i]) inside[id]++ }
        val isGraphic = BooleanArray(size = labels.size) { id ->
            val dust = labels.counts[id] < DUST && keptLines.none { it.box.intersects(labels.boxes[id]) }
            !dust && inside[id] * 2 < labels.counts[id]
        }
        val graphics = Mask(width = masks.width, height = masks.height)
        labels.ids.forEachIndexed { i, id ->
            graphics.bits[i] = (id >= 0 && isGraphic[id]) || masks.rules.bits[i]
        }
        // Letters touching a table rule must not end up twice: inside replaced words only the rule stays.
        runBoxes.forEach { box ->
            for (y in maxOf(0, box.y) until minOf(masks.height, box.y + box.height)) {
                for (x in maxOf(0, box.x) until minOf(masks.width, box.x + box.width)) {
                    if (!masks.rules[x, y]) graphics[x, y] = false
                }
            }
        }
        return graphics
    }

    private fun filledShare(keep: Mask, box: Rectangle): Double {
        var kept = 0
        for (y in box.y until box.y + box.height) {
            for (x in box.x until box.x + box.width) {
                if (keep[x, y]) kept++
            }
        }
        return kept.toDouble() / (box.width * box.height)
    }

    private fun cut(masks: PageMasks, box: Rectangle, keep: Mask): BufferedImage {
        val colourful = isColourful(masks = masks, box = box, keep = keep)
        val type = if (colourful) BufferedImage.TYPE_INT_RGB else BufferedImage.TYPE_BYTE_GRAY
        val image = BufferedImage(box.width, box.height, type)
        for (y in 0 until box.height) {
            for (x in 0 until box.width) {
                val px = box.x + x
                val py = box.y + y
                if (colourful) {
                    image.setRGB(x, y, if (keep[px, py]) masks.page.getRGB(px, py) else WHITE_RGB)
                } else {
                    // Written to the raster directly: setRGB on a grey image converts to linear grey
                    // and darkens mid-tones.
                    image.raster.setSample(x, y, 0, if (keep[px, py]) masks.gray[py * masks.width + px] else WHITE)
                }
            }
        }
        return image
    }

    private fun isColourful(masks: PageMasks, box: Rectangle, keep: Mask): Boolean {
        for (y in box.y until box.y + box.height) {
            for (x in box.x until box.x + box.width) {
                if (keep[x, y] && spread(rgb = masks.page.getRGB(x, y)) > COLOUR_SPREAD) return true
            }
        }
        return false
    }

    private fun spread(rgb: Int): Int {
        val red = rgb shr RED_SHIFT and BYTE
        val green = rgb shr GREEN_SHIFT and BYTE
        val blue = rgb and BYTE
        return maxOf(abs(red - green), abs(green - blue), abs(red - blue))
    }
}
