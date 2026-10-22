package dms.storage.catalog

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

    /** [dto] with the settings of [settings] filled in. */
    fun withSettings(dto: CatalogDto, settings: CatalogSettingsModel): CatalogDto {
        return dto.copy(
            fieldDefinitions = settings.fieldDefinitions.map { FieldMapper.toDto(definition = it) },
            savedSearches = settings.savedSearches.map { QueryMapper.toDto(search = it) },
            rules = settings.rules.map { RuleMapper.toDto(rule = it) },
            correspondents = settings.correspondents
                .sortedBy { it.name.lowercase(Locale.ROOT) }
                .map { CorrespondentDto(name = it.name, aliases = it.aliases) },
            layout = toDto(layout = settings.layout),
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

    /** `null` for the default layout, so a library that uses none of it writes nothing. */
    private fun toDto(layout: LibraryLayoutModel): LayoutDto? {
        if (layout == LibraryLayoutModel()) {
            return null
        }
        val inbox = layout.inbox
        return LayoutDto(
            inbox = InboxDto(
                enabled = inbox.enabled,
                path = inbox.path.value,
                aiAnalysis = inbox.aiAnalysis,
                aiTargetFolder = inbox.aiTargetFolder,
                autoAnalyze = inbox.autoAnalyze,
            ).takeIf { inbox != InboxSettingsModel() },
            pathTemplate = layout.pathTemplate,
            defaultCurrency = layout.defaultCurrency?.value,
            embedMetadata = layout.embedMetadata.takeIf { it },
        )
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
