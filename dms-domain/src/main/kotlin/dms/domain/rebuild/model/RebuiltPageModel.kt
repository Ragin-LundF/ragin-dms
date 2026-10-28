package dms.domain.rebuild.model

import dms.domain.rebuild.KeepReason

/**
 * The result for one page: [linesAsText] of [linesTotal] lines became real text, the page was
 * straightened by [angle] degrees, and [keptReasons] counts why the other lines stayed images.
 */
data class RebuiltPageModel(
    val linesAsText: Int,
    val linesTotal: Int,
    val angle: Double,
    val keptReasons: Map<KeepReason, Int>,
    val regions: List<RebuildRegionModel>,
)
