package dms.domain.deadline.model

import dms.domain.document.model.DocumentIdModel
import dms.domain.field.model.FieldKeyModel
import java.time.LocalDate

/** A date ahead for one document: a deadline [field], or its retention end when [field] is `null`. */
data class UpcomingDateModel(
    val documentId: DocumentIdModel,
    val field: FieldKeyModel?,
    val date: LocalDate,
)
