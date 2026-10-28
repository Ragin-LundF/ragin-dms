package dms.domain.rebuild.model

/** An area that is no printed text: a logo, stamp, signature, photo or code, with the AI's [kind]. */
data class LayoutGraphicModel(
    val kind: String,
    val box: LayoutBoxModel,
)
