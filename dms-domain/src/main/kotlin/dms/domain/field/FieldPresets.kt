package dms.domain.field

import dms.domain.field.model.FieldDefinitionModel
import dms.domain.field.model.FieldKeyModel

/** The fields a library gets when it uses custom fields for the first time. */
object FieldPresets {

    val AMOUNT: FieldDefinitionModel = FieldDefinitionModel(
        key = FieldKeyModel(value = "amount"),
        label = "Amount",
        type = FieldType.MONEY,
    )
    val DUE_DATE: FieldDefinitionModel = FieldDefinitionModel(
        key = FieldKeyModel(value = "due"),
        label = "Due date",
        type = FieldType.DATE,
        role = FieldRole.DUE_DATE,
    )
    val CANCEL_BY: FieldDefinitionModel = FieldDefinitionModel(
        key = FieldKeyModel(value = "cancel-by"),
        label = "Cancel by",
        type = FieldType.DATE,
        role = FieldRole.CANCEL_BY,
    )
    val CONTRACT_END: FieldDefinitionModel = FieldDefinitionModel(
        key = FieldKeyModel(value = "contract-end"),
        label = "Contract end",
        type = FieldType.DATE,
        role = FieldRole.CONTRACT_END,
    )
    val WARRANTY_END: FieldDefinitionModel = FieldDefinitionModel(
        key = FieldKeyModel(value = "warranty-until"),
        label = "Warranty until",
        type = FieldType.DATE,
        role = FieldRole.WARRANTY_END,
    )

    val ALL: List<FieldDefinitionModel> = listOf(AMOUNT, DUE_DATE, CANCEL_BY, CONTRACT_END, WARRANTY_END)
}
