package dms.domain.rebuild.model

/** What the AI found on one page: the text lines in reading order and the graphics. */
data class PageLayoutModel(
    val lines: List<LayoutLineModel>,
    val graphics: List<LayoutGraphicModel>,
)
