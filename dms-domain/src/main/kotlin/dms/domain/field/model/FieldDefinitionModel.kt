package dms.domain.field.model

import dms.domain.field.FieldRole
import dms.domain.field.FieldType

/**
 * A custom field the library uses, such as "Amount" or "Cancel by". Only date fields can carry a
 * deadline [role].
 */
data class FieldDefinitionModel(
    val key: FieldKeyModel,
    val label: String,
    val type: FieldType,
    val role: FieldRole? = null,
) {
    init {
        require(value = label.isNotBlank()) { "Field label must not be blank" }
        require(value = role == null || type == FieldType.DATE) { "Only date fields can be deadlines" }
    }
}
