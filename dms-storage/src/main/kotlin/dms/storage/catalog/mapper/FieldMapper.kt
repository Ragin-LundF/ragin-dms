package dms.storage.catalog.mapper

import dms.domain.field.FieldRole
import dms.domain.field.FieldType
import dms.domain.field.FieldValue
import dms.domain.field.model.CurrencyCodeModel
import dms.domain.field.model.FieldDefinitionModel
import dms.domain.field.model.FieldKeyModel
import dms.storage.catalog.dto.FieldValueDto
import dms.storage.catalog.dto.settings.FieldDefinitionDto
import java.time.LocalDate

/** Maps custom field definitions and values. Numbers are written as plain decimal strings, never as floats. */
object FieldMapper {

    fun toDto(definition: FieldDefinitionModel): FieldDefinitionDto {
        return FieldDefinitionDto(
            key = definition.key.value,
            label = definition.label,
            type = definition.type.name,
            role = definition.role?.name,
        )
    }

    fun toDomain(definition: FieldDefinitionDto): FieldDefinitionModel {
        return FieldDefinitionModel(
            key = FieldKeyModel(value = definition.key),
            label = definition.label,
            type = FieldType.valueOf(definition.type),
            role = definition.role?.let { FieldRole.valueOf(it) },
        )
    }

    /** The values sorted by key, so the file is stable. */
    fun toDto(fields: Map<FieldKeyModel, FieldValue>): Map<String, FieldValueDto> {
        return fields.entries.sortedBy { it.key.value }.associate { (key, value) -> key.value to toDto(value = value) }
    }

    fun toDomain(fields: Map<String, FieldValueDto>): Map<FieldKeyModel, FieldValue> {
        return fields.entries.associate { (key, value) -> FieldKeyModel(value = key) to toDomain(value = value) }
    }

    fun toDto(value: FieldValue): FieldValueDto {
        return when (value) {
            is FieldValue.Text -> FieldValueDto(text = value.text)
            is FieldValue.Number -> FieldValueDto(number = value.number.toPlainString())
            is FieldValue.Money -> FieldValueDto(amount = value.amount.toPlainString(), currency = value.currency.value)
            is FieldValue.Date -> FieldValueDto(date = value.date.toString())
            is FieldValue.Flag -> FieldValueDto(flag = value.flag)
        }
    }

    /** Throws [IllegalArgumentException] for a value that sets no or more than one type. */
    fun toDomain(value: FieldValueDto): FieldValue {
        val candidates = listOfNotNull(
            value.text?.let { FieldValue.Text(text = it) },
            value.number?.let { FieldValue.Number(number = it.toBigDecimal()) },
            value.amount?.let { amount ->
                FieldValue.Money(
                    amount = amount.toBigDecimal(),
                    currency = CurrencyCodeModel(
                        value = requireNotNull(value = value.currency) { "Amount without currency" },
                    ),
                )
            },
            value.date?.let { FieldValue.Date(date = LocalDate.parse(it)) },
            value.flag?.let { FieldValue.Flag(flag = it) },
        )
        require(value = candidates.size == 1) { "A field value must have exactly one type" }
        return candidates.single()
    }
}
