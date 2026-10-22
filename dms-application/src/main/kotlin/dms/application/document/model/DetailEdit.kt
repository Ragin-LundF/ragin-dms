package dms.application.document.model

import dms.domain.document.model.DocumentIdModel
import dms.domain.field.FieldValue
import dms.domain.field.model.FieldKeyModel
import dms.domain.retention.model.RetentionModel
import dms.domain.taxonomy.model.CategoryIdModel

/** A change to a document's details that touches no file: custom fields, the note, retention and links. */
sealed interface DetailEdit {

    /** Replaces the custom field values of a document. */
    data class SetFields(val documentId: DocumentIdModel, val fields: Map<FieldKeyModel, FieldValue>) : DetailEdit

    /** Sets the note; a blank note removes it. */
    data class SetNote(val documentId: DocumentIdModel, val note: String?) : DetailEdit

    /** Sets the opt-in keep-until date of a document, or removes it with `null`. */
    data class SetRetention(val documentId: DocumentIdModel, val retention: RetentionModel?) : DetailEdit

    /** Sets the opt-in retention period of a category in years, or removes it with `null`. */
    data class SetRetentionYears(val categoryId: CategoryIdModel, val years: Int?) : DetailEdit

    data class Link(val first: DocumentIdModel, val second: DocumentIdModel, val label: String?) : DetailEdit

    data class Unlink(val first: DocumentIdModel, val second: DocumentIdModel) : DetailEdit
}
