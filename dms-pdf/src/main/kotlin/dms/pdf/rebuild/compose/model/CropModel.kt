package dms.pdf.rebuild.compose.model

import java.awt.Rectangle
import java.awt.image.BufferedImage

/**
 * A piece of the scan that stays an image: a logo, stamp, signature, rule, or a line that was not
 * replaced. [filled] is the share of the box that holds scan pixels; large filled crops are shaded areas or
 * photos, not line art.
 */
class CropModel(
    val box: Rectangle,
    val image: BufferedImage,
    val filled: Double,
)
