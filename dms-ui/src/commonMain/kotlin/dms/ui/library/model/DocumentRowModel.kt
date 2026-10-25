package dms.ui.library.model

import dms.domain.document.model.DocumentModel
import dms.domain.field.FieldValue
import java.time.LocalDate

/**
 * One document as the grid and the list show it: its category's name, and the [amount] and [dueDate]
 * from its custom fields, if it has them.
 */
data class DocumentRowModel(
    val document: DocumentModel,
    val categoryName: String?,
    val amount: FieldValue.Money? = null,
    val dueDate: LocalDate? = null,
)
