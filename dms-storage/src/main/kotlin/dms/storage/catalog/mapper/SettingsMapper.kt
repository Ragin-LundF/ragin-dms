package dms.storage.catalog.mapper

import dms.domain.field.model.CurrencyCodeModel
import dms.domain.library.model.RelativePathModel
import dms.domain.rule.model.CorrespondentModel
import dms.domain.settings.model.CatalogSettingsModel
import dms.domain.settings.model.InboxSettingsModel
import dms.domain.settings.model.LibraryLayoutModel
import dms.storage.catalog.dto.CatalogDto
import dms.storage.catalog.dto.settings.CorrespondentDto
import dms.storage.catalog.dto.settings.InboxDto
import dms.storage.catalog.dto.settings.LayoutDto
import java.util.Locale

/**
 * Maps the library-wide settings. Field definitions, saved searches and rules keep the order the user
 * gave them; correspondents are sorted by name.
 */
object SettingsMapper {

    /**
     * [dto] with the settings of [settings] filled in; fields only newer versions know are taken over
     * from [previous], per field key, id or name.
     */
    fun withSettings(dto: CatalogDto, settings: CatalogSettingsModel, previous: CatalogDto?): CatalogDto {
        val searches = settings.savedSearches.map { QueryMapper.toDto(search = it) }
        val previousSearches = previous?.savedSearches.orEmpty().associateBy { it.id }
        searches.forEach { UnknownFields.carry(entry = it.query, previous = previousSearches[it.id]?.query) }
        val rules = settings.rules.map { RuleMapper.toDto(rule = it) }
        val previousRules = previous?.rules.orEmpty().associateBy { it.id }
        rules.forEach { UnknownFields.carry(entry = it.actions, previous = previousRules[it.id]?.actions) }
        val correspondents = settings.correspondents
            .sortedBy { it.name.lowercase(Locale.ROOT) }
            .map { CorrespondentDto(name = it.name, aliases = it.aliases) }
        return dto.copy(
            fieldDefinitions = UnknownFields.carry(
                entries = settings.fieldDefinitions.map { FieldMapper.toDto(definition = it) },
                previous = previous?.fieldDefinitions.orEmpty(),
            ) { it.key },
            savedSearches = UnknownFields.carry(
                entries = searches,
                previous = previous?.savedSearches.orEmpty(),
            ) { it.id },
            rules = UnknownFields.carry(entries = rules, previous = previous?.rules.orEmpty()) { it.id },
            correspondents = UnknownFields.carry(
                entries = correspondents,
                previous = previous?.correspondents.orEmpty(),
            ) { it.name },
            layout = toDto(layout = settings.layout, previous = previous?.layout),
        )
    }

    fun toDomain(dto: CatalogDto): CatalogSettingsModel {
        return CatalogSettingsModel(
            fieldDefinitions = dto.fieldDefinitions.map { FieldMapper.toDomain(definition = it) },
            savedSearches = dto.savedSearches.map { QueryMapper.toDomain(search = it) },
            rules = dto.rules.map { RuleMapper.toDomain(rule = it) },
            correspondents = dto.correspondents.map { CorrespondentModel(name = it.name, aliases = it.aliases) },
            layout = dto.layout?.let { toDomain(layout = it) } ?: LibraryLayoutModel(),
        )
    }

    /**
     * `null` for the default layout, so a library that uses none of it writes nothing, unless the
     * [previous] file holds fields of a newer version there.
     */
    private fun toDto(layout: LibraryLayoutModel, previous: LayoutDto?): LayoutDto? {
        if (layout == LibraryLayoutModel() && !UnknownFields.hasAny(entry = previous) &&
            !UnknownFields.hasAny(entry = previous?.inbox)
        ) {
            return null
        }
        val inbox = layout.inbox
        val inboxDto = InboxDto(
            enabled = inbox.enabled,
            path = inbox.path.value,
            aiAnalysis = inbox.aiAnalysis,
            aiTargetFolder = inbox.aiTargetFolder,
            autoAnalyze = inbox.autoAnalyze,
        ).takeIf { inbox != InboxSettingsModel() || UnknownFields.hasAny(entry = previous?.inbox) }
        val dto = LayoutDto(
            inbox = UnknownFields.carry(entry = inboxDto, previous = previous?.inbox),
            pathTemplate = layout.pathTemplate,
            defaultCurrency = layout.defaultCurrency?.value,
            embedMetadata = layout.embedMetadata.takeIf { it },
        )
        return UnknownFields.carry(entry = dto, previous = previous)
    }

    private fun toDomain(layout: LayoutDto): LibraryLayoutModel {
        val inbox = layout.inbox?.let {
            InboxSettingsModel(
                enabled = it.enabled,
                path = RelativePathModel.of(raw = it.path),
                aiAnalysis = it.aiAnalysis,
                aiTargetFolder = it.aiTargetFolder,
                autoAnalyze = it.autoAnalyze,
            )
        }
        return LibraryLayoutModel(
            inbox = inbox ?: InboxSettingsModel(),
            pathTemplate = layout.pathTemplate,
            defaultCurrency = layout.defaultCurrency?.let { CurrencyCodeModel(value = it) },
            embedMetadata = layout.embedMetadata == true,
        )
    }
}
