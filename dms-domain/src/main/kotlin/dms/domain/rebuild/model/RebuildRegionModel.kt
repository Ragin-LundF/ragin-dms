package dms.domain.rebuild.model

import dms.domain.rebuild.RegionKind

/** A region of a rebuilt page and what became of it. */
data class RebuildRegionModel(
    val box: LayoutBoxModel,
    val kind: RegionKind,
)
