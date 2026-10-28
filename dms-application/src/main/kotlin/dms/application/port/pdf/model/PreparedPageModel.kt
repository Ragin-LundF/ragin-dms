package dms.application.port.pdf.model

import dms.application.port.ai.model.PageImageModel

/** Page [pageIndex] of a scan, rendered at full resolution and straightened by [angle] degrees, as PNG. */
class PreparedPageModel(
    val pageIndex: Int,
    val image: PageImageModel,
    val angle: Double,
)
