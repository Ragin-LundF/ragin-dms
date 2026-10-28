package dms.pdf.rebuild.image

import java.awt.Rectangle

/**
 * The connected components of a mask in the 8-neighbourhood, found with union-find instead of recursion.
 * [ids] holds the component of every pixel, or -1; [counts] and [boxes] are per component.
 */
class Labels private constructor(val ids: IntArray, val counts: IntArray, val boxes: List<Rectangle>) {

    val size: Int get() = counts.size

    companion object {

        fun of(mask: Mask): Labels {
            val parent = IntArray(size = mask.bits.size) { -1 }
            link(mask = mask, parent = parent)
            return collect(mask = mask, parent = parent)
        }

        private fun link(mask: Mask, parent: IntArray) {
            val width = mask.width
            for (y in 0 until mask.height) {
                for (x in 0 until width) {
                    if (mask[x, y]) linkPixel(mask = mask, parent = parent, x = x, y = y)
                }
            }
        }

        /** Joins a set pixel with its set neighbours to the left and in the row above. */
        private fun linkPixel(mask: Mask, parent: IntArray, x: Int, y: Int) {
            val width = mask.width
            val i = y * width + x
            parent[i] = i
            if (x > 0 && mask[x - 1, y]) union(parent = parent, a = i, b = i - 1)
            if (y == 0) return
            if (mask[x, y - 1]) union(parent = parent, a = i, b = i - width)
            if (x > 0 && mask[x - 1, y - 1]) union(parent = parent, a = i, b = i - width - 1)
            if (x < width - 1 && mask[x + 1, y - 1]) union(parent = parent, a = i, b = i - width + 1)
        }

        private fun collect(mask: Mask, parent: IntArray): Labels {
            val ids = IntArray(size = parent.size) { -1 }
            val byRoot = HashMap<Int, Int>()
            val counts = ArrayList<Int>()
            val left = ArrayList<Int>()
            val top = ArrayList<Int>()
            val right = ArrayList<Int>()
            val bottom = ArrayList<Int>()
            for (i in parent.indices) {
                if (parent[i] < 0) continue
                val x = i % mask.width
                val y = i / mask.width
                val id = byRoot.getOrPut(key = find(parent = parent, i = i)) {
                    counts += 0
                    left += x
                    top += y
                    right += x
                    bottom += y
                    counts.size - 1
                }
                counts[id]++
                left[id] = minOf(left[id], x)
                right[id] = maxOf(right[id], x)
                bottom[id] = y
                ids[i] = id
            }
            val boxes = counts.indices.map { id ->
                Rectangle(left[id], top[id], right[id] - left[id] + 1, bottom[id] - top[id] + 1)
            }
            return Labels(ids = ids, counts = counts.toIntArray(), boxes = boxes)
        }

        private fun find(parent: IntArray, i: Int): Int {
            var root = i
            while (parent[root] != root) root = parent[root]
            var node = i
            while (parent[node] != root) {
                val next = parent[node]
                parent[node] = root
                node = next
            }
            return root
        }

        private fun union(parent: IntArray, a: Int, b: Int) {
            val rootA = find(parent = parent, i = a)
            val rootB = find(parent = parent, i = b)
            if (rootA != rootB) parent[maxOf(rootA, rootB)] = minOf(rootA, rootB)
        }
    }
}
