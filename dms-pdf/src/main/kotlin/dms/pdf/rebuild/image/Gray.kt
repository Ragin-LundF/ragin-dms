package dms.pdf.rebuild.image

import java.awt.image.BufferedImage

/** Grey values and the Otsu threshold that separates ink from paper. */
object Gray {

    private const val LEVELS = 256
    private const val RED_WEIGHT = 299
    private const val GREEN_WEIGHT = 587
    private const val BLUE_WEIGHT = 114
    private const val WEIGHTS = 1000
    private const val BYTE = 0xFF
    private const val RED_SHIFT = 16
    private const val GREEN_SHIFT = 8

    /** The grey value 0..255 of every pixel of [image], row by row. */
    fun of(image: BufferedImage): IntArray {
        val rgb = image.getRGB(0, 0, image.width, image.height, null, 0, image.width)
        return IntArray(size = rgb.size) { valueOf(rgb = rgb[it]) }
    }

    fun valueOf(rgb: Int): Int {
        val red = rgb shr RED_SHIFT and BYTE
        val green = rgb shr GREEN_SHIFT and BYTE
        val blue = rgb and BYTE
        return (red * RED_WEIGHT + green * GREEN_WEIGHT + blue * BLUE_WEIGHT) / WEIGHTS
    }

    /** Otsu's threshold: the grey value that best splits [gray] into two classes. */
    fun otsu(gray: IntArray): Int {
        val histogram = IntArray(size = LEVELS)
        gray.forEach { histogram[it]++ }
        val total = gray.size.toDouble()
        val sum = (0 until LEVELS).sumOf { it.toDouble() * histogram[it] }
        var sumBackground = 0.0
        var weightBackground = 0.0
        var best = 0.0
        var threshold = LEVELS / 2
        for (t in 0 until LEVELS) {
            weightBackground += histogram[t]
            val weightForeground = total - weightBackground
            sumBackground += t.toDouble() * histogram[t]
            if (weightBackground == 0.0 || weightForeground == 0.0) continue
            val difference = sumBackground / weightBackground - (sum - sumBackground) / weightForeground
            val between = weightBackground * weightForeground * difference * difference
            if (between > best) {
                best = between
                threshold = t
            }
        }
        return threshold
    }
}
