package dms.domain.restructure.template

import dms.domain.field.model.FieldKeyModel

/** A piece of one folder name in a [PathTemplateModel]: fixed text or a placeholder. */
sealed interface TemplatePart {

    data class Literal(val text: String) : TemplatePart

    /** A value of the document; [fallback] is used when the document has none. */
    data class Placeholder(
        val name: PlaceholderName,
        val fieldKey: FieldKeyModel? = null,
        val fallback: String? = null,
    ) : TemplatePart {
        init {
            require(value = (name == PlaceholderName.FIELD) == (fieldKey != null)) { "Only {field:…} takes a key" }
        }
    }
}
