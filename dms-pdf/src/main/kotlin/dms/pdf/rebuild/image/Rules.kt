package dms.pdf.rebuild.image

/**
 * Table rules and underlines: pixels in a horizontal or vertical run of at least [RUN] pixels that is also
 * thin (at most [THICK] pixels across), so big or bold letters are not rules.
 */
object Rules {

    /** About 8 mm at 300 dpi: longer than the strokes of normal letters, shorter than table lines. */
    const val RUN: Int = 100
    private const val THICK = 8
    private const val SAMPLES = 5

    fun of(mask: Mask): Mask {
        val result = Mask(width = mask.width, height = mask.height)
        for (y in 0 until mask.height) {
            runs(length = mask.width, isOn = { mask[it, y] }) { from, to ->
                if (isThin(mask = mask, from = from, to = to, at = y, horizontal = true)) {
                    for (x in from until to) result[x, y] = true
                }
            }
        }
        for (x in 0 until mask.width) {
            runs(length = mask.height, isOn = { mask[x, it] }) { from, to ->
                if (isThin(mask = mask, from = from, to = to, at = x, horizontal = false)) {
                    for (y in from until to) result[x, y] = true
                }
            }
        }
        return result
    }

    /** Calls [found] with every run of set pixels that is at least [RUN] long. */
    private inline fun runs(length: Int, isOn: (Int) -> Boolean, found: (Int, Int) -> Unit) {
        var start = -1
        for (i in 0..length) {
            val on = i < length && isOn(i)
            if (on && start < 0) start = i
            if (!on && start >= 0) {
                if (i - start >= RUN) found(start, i)
                start = -1
            }
        }
    }

    /** The median thickness across the run, sampled at five points along it. */
    private fun isThin(mask: Mask, from: Int, to: Int, at: Int, horizontal: Boolean): Boolean {
        val thickness = (1..SAMPLES).map { i ->
            val along = from + (to - from) * i / (SAMPLES + 1)
            if (horizontal) {
                thicknessAt(limit = mask.height, at = at) { mask[along, it] }
            } else {
                thicknessAt(limit = mask.width, at = at) { mask[it, along] }
            }
        }.sorted()
        return thickness[SAMPLES / 2] <= THICK
    }

    private inline fun thicknessAt(limit: Int, at: Int, isOn: (Int) -> Boolean): Int {
        var before = 0
        var after = 0
        while (at - before - 1 >= 0 && isOn(at - before - 1) && before <= THICK) before++
        while (at + after + 1 < limit && isOn(at + after + 1) && after <= THICK) after++
        return before + after + 1
    }
}
