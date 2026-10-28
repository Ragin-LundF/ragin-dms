package dms.pdf.rebuild

import dms.domain.rebuild.model.LayoutBoxModel
import java.awt.Rectangle

/** Converts boxes between 0..1000 of a page and pixels of its [width] x [height] image. */
class PixelBoxes(private val width: Int, private val height: Int) {

    /** The pixels of [box], at least one pixel and inside the image. */
    fun toPixels(box: LayoutBoxModel): Rectangle {
        val x0 = pixels(value = box.left, size = width).coerceIn(minimumValue = 0, maximumValue = width - 1)
        val y0 = pixels(value = box.top, size = height).coerceIn(minimumValue = 0, maximumValue = height - 1)
        val x1 = pixels(value = box.right, size = width).coerceIn(minimumValue = x0 + 1, maximumValue = width)
        val y1 = pixels(value = box.bottom, size = height).coerceIn(minimumValue = y0 + 1, maximumValue = height)
        return Rectangle(x0, y0, x1 - x0, y1 - y0)
    }

    private fun pixels(value: Double, size: Int): Int {
        return (value * size / LayoutBoxModel.SCALE).toInt()
    }

    fun toLayout(box: Rectangle): LayoutBoxModel {
        return LayoutBoxModel(
            left = box.x * LayoutBoxModel.SCALE / width,
            top = box.y * LayoutBoxModel.SCALE / height,
            right = (box.x + box.width) * LayoutBoxModel.SCALE / width,
            bottom = (box.y + box.height) * LayoutBoxModel.SCALE / height,
        )
    }
}
