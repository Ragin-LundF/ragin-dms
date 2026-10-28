package dms.pdf.rebuild.image

import java.awt.Rectangle

/** A boolean image the size of the page, row by row in [bits]. */
class Mask(val width: Int, val height: Int, val bits: BooleanArray = BooleanArray(size = width * height)) {

    operator fun get(x: Int, y: Int): Boolean {
        return bits[y * width + x]
    }

    operator fun set(x: Int, y: Int, value: Boolean) {
        bits[y * width + x] = value
    }

    /** Square dilation by [radius], as two running-count passes, so the cost does not grow with the radius. */
    fun dilate(radius: Int): Mask {
        val horizontal = BooleanArray(size = bits.size)
        for (y in 0 until height) {
            slide(length = width, radius = radius, isOn = { bits[y * width + it] }) { x, on ->
                horizontal[y * width + x] = on
            }
        }
        val result = BooleanArray(size = bits.size)
        for (x in 0 until width) {
            slide(length = height, radius = radius, isOn = { horizontal[it * width + x] }) { y, on ->
                result[y * width + x] = on
            }
        }
        return Mask(width = width, height = height, bits = result)
    }

    fun fill(box: Rectangle, value: Boolean = true) {
        for (y in maxOf(0, box.y) until minOf(height, box.y + box.height)) {
            for (x in maxOf(0, box.x) until minOf(width, box.x + box.width)) {
                bits[y * width + x] = value
            }
        }
    }

    /** Whether any pixel in [box] is set. */
    fun any(box: Rectangle): Boolean {
        for (y in maxOf(0, box.y) until minOf(height, box.y + box.height)) {
            for (x in maxOf(0, box.x) until minOf(width, box.x + box.width)) {
                if (bits[y * width + x]) {
                    return true
                }
            }
        }
        return false
    }

    /** A running count over one row or column: [emit] gets whether a set pixel lies within [radius]. */
    private inline fun slide(length: Int, radius: Int, isOn: (Int) -> Boolean, emit: (Int, Boolean) -> Unit) {
        var count = 0
        for (i in 0 until minOf(radius, length)) {
            if (isOn(i)) count++
        }
        for (i in 0 until length) {
            if (i + radius < length && isOn(i + radius)) count++
            if (i - radius - 1 >= 0 && isOn(i - radius - 1)) count--
            emit(i, count > 0)
        }
    }

    companion object {
        /** The pixels set in [first] and not in [second]. */
        fun without(first: Mask, second: Mask): Mask {
            val bits = BooleanArray(size = first.bits.size) { first.bits[it] && !second.bits[it] }
            return Mask(width = first.width, height = first.height, bits = bits)
        }
    }
}
