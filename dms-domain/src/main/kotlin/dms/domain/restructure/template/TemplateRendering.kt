package dms.domain.restructure.template

import dms.domain.library.model.RelativePathModel

/** The folder a [PathTemplateModel] gives one document. */
sealed interface TemplateRendering {

    data class Rendered(val folder: RelativePathModel) : TemplateRendering

    /** The document has no value for [placeholder] and the template gives no fallback. */
    data class Missing(val placeholder: PlaceholderName) : TemplateRendering
}
